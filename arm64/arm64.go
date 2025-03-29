package arm64

import (
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"math"
)

type InstructionType uint

const (
	ADDImm InstructionType = iota
	SUBImm
	MOVZ
	ADR
	LDRImm
	STRImm
	B
	BL
	BCond
	RET
	NOP
)

type Register struct {
	Num  int  `json:"num"`
	Is64 bool `json:"is64"`
}

type Immediate struct {
	Value int64 `json:"value"`
	Shift uint  `json:"shift,omitempty"`
}

type Label string

type ConditionCode uint

const (
	EQ ConditionCode = 0x0
	NE ConditionCode = 0x1
	CS ConditionCode = 0x2
	HS ConditionCode = 0x2
	CC ConditionCode = 0x3
	LO ConditionCode = 0x3
	MI ConditionCode = 0x4
	PL ConditionCode = 0x5
	VS ConditionCode = 0x6
	VC ConditionCode = 0x7
	HI ConditionCode = 0x8
	LS ConditionCode = 0x9
	GE ConditionCode = 0xA
	LT ConditionCode = 0xB
	GT ConditionCode = 0xC
	LE ConditionCode = 0xD
	AL ConditionCode = 0xE
	NV ConditionCode = 0xF
)

type ImmediateKind int

const (
	ImmKindNone ImmediateKind = iota
	ImmKindArithmetic
	ImmKindMoveWide
	ImmKindOffset // Unscaled offset (e.g., LDR/STR byte offset, ADR byte offset)
	ImmKindBranch // Scaled offset (instruction count)
)

type MemOperation int

const (
	MemOpNone MemOperation = iota
	MemOpLoad
	MemOpStore
)

type LogicalInstruction struct {
	Opcode InstructionType

	Rd *Register
	Rn *Register
	Rm *Register
	Rt *Register // Typically Rd or the data register for Ld/St

	ImmValue int64
	ImmKind  ImmediateKind
	ImmShift uint // For ImmKindArithmetic shift=12, MoveWide hw(0-3)

	MemOp   MemOperation
	MemSize uint // 8, 16, 32, 64, 128 (only 32/64 used currently)

	Condition ConditionCode
	IsLink    bool

	SetFlags bool
	// OpSize implicitly derived from Rd/Rt register size in encoding phase
}

type Node interface {
	json.Marshaler
	json.Unmarshaler
	EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error)
	InstructionType() InstructionType
}

type Program struct {
	Instructions []Node         `json:"instructions"`
	Labels       map[string]int `json:"labels"`
}

type BaseInstruction struct {
	IType InstructionType `json:"type"`
}

func (b *BaseInstruction) InstructionType() InstructionType {
	return b.IType
}

type DataProcessingImmediate struct {
	BaseInstruction
	DestReg   Register  `json:"dest_reg"`
	SourceReg Register  `json:"source_reg"`
	Imm       Immediate `json:"imm"`
	SetFlags  bool      `json:"set_flags,omitempty"`
}

type MoveWideImmediate struct {
	BaseInstruction
	DestReg Register `json:"dest_reg"`
	Imm     uint16   `json:"imm"`
	HW      uint     `json:"hw"`
}

type PCRelativeAddressing struct {
	BaseInstruction
	DestReg Register `json:"dest_reg"`
	Target  Label    `json:"target"`
}

type LoadStoreImmediate struct {
	BaseInstruction
	DataReg Register  `json:"data_reg"`
	BaseReg Register  `json:"base_reg"`
	Offset  Immediate `json:"offset"`
	IsLoad  bool      `json:"is_load"`
}

type UnconditionalBranchImmediate struct {
	BaseInstruction
	Target Label `json:"target"`
	IsLink bool  `json:"is_link"`
}

type ConditionalBranchImmediate struct {
	BaseInstruction
	Target    Label         `json:"target"`
	Condition ConditionCode `json:"condition"`
}

type UnconditionalBranchRegister struct {
	BaseInstruction
	TargetReg Register `json:"target_reg"`
}

type NoOperandInstruction struct {
	BaseInstruction
}

func (r Register) MarshalJSON() ([]byte, error) {
	prefix := "w"
	if r.Is64 {
		prefix = "x"
	}
	var name string
	if r.Is64 && r.Num == 31 {
		name = "sp"
	} else if !r.Is64 && r.Num == 31 {
		name = "wsp"
	} else if r.Num == 31 {
		name = prefix + "zr"
	} else {
		name = fmt.Sprintf("%s%d", prefix, r.Num)
	}
	return json.Marshal(name)
}

func (r *Register) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return err
	}
	if len(s) < 2 {
		return fmt.Errorf("invalid register name: %s", s)
	}

	switch s {
	case "sp":
		r.Num = 31
		r.Is64 = true
		return nil
	case "wsp":
		r.Num = 31
		r.Is64 = false
		return nil
	case "xzr":
		r.Num = 31
		r.Is64 = true
		return nil
	case "wzr":
		r.Num = 31
		r.Is64 = false
		return nil
	case "lr":
		r.Num = 30
		r.Is64 = true
		return nil
	}

	prefix := s[0]
	numStr := s[1:]

	switch prefix {
	case 'x':
		r.Is64 = true
	case 'w':
		r.Is64 = false
	default:
		return fmt.Errorf("invalid register prefix: %c in %s", prefix, s)
	}

	if _, err := fmt.Sscan(numStr, &r.Num); err != nil {
		return fmt.Errorf("invalid register number: %s in %s", numStr, s)
	}

	if r.Num < 0 || r.Num > 30 {
		return fmt.Errorf("register number out of range (0-30): %d", r.Num)
	}
	return nil
}

func encodeReg(r *Register) (uint32, error) {
	if r == nil {
		return 0, errors.New("attempted to encode nil register")
	}
	return uint32(r.Num & 0x1F), nil
}

func getRegSize(r *Register) (uint, error) {
	if r == nil {
		return 0, errors.New("attempted to get size of nil register")
	}
	if r.Is64 {
		return 64, nil
	}
	return 32, nil
}

func encodeImm12ShiftForLogical(li LogicalInstruction) (uint32, error) {
	val := li.ImmValue
	shift := li.ImmShift

	if shift != 0 && shift != 12 {
		return 0, fmt.Errorf("invalid shift for immediate: %d (must be 0 or 12)", shift)
	}
	if val < 0 || val > 0xFFF {
		return 0, fmt.Errorf("immediate value out of range (0-4095): %d", val)
	}

	imm12 := uint32(val & 0xFFF)
	sh := uint32(0)
	if shift == 12 {
		sh = 1
	}

	return (sh << 22) | (imm12 << 10), nil
}

func encodeLoadStoreImmOffsetForLogical(li LogicalInstruction) (uint32, error) {
	var sizeBits uint32
	opSize, err := getRegSize(li.Rt)
	if err != nil {
		return 0, fmt.Errorf("missing Rt register for load/store size: %w", err)
	}

	switch opSize {
	case 32:
		sizeBits = 0b10
	case 64:
		sizeBits = 0b11
	default:
		return 0, fmt.Errorf("unsupported load/store size: %d", opSize)
	}

	scaleFactor := int64(1) << (opSize / 16) // 4 for 32, 8 for 64

	if li.ImmValue < 0 || li.ImmValue >= (4096*scaleFactor) || li.ImmValue%scaleFactor != 0 {
		return 0, fmt.Errorf("load/store immediate offset out of range [0, %d) or not multiple of %d: %d", 4096*scaleFactor, scaleFactor, li.ImmValue)
	}
	imm12 := uint32(li.ImmValue / scaleFactor)
	return imm12 << 10, nil
}

func encodePCRelOffsetForLogical(li LogicalInstruction) (uint32, error) {
	offset := li.ImmValue // Assumes ImmValue already holds the calculated byte offset

	if offset < -(1 << 20) || offset >= (1 << 20) {
		return 0, fmt.Errorf("ADR offset out of range (-1MB to +1MB): %d bytes", offset)
	}

	immlo := uint32((offset & 0x3) << 29)
	immhi := uint32(((offset >> 2) & 0x7FFFF) << 5)

	return immlo | immhi, nil
}

func encodeBranchOffsetForLogical(li LogicalInstruction, bitSize uint) (uint32, error) {
	offset := li.ImmValue // Assumes ImmValue already holds the calculated instruction offset

	var minOffset, maxOffset int64
	var mask uint32

	switch bitSize {
	case 19:
		minOffset, maxOffset, mask = -(1 << 18), (1 << 18)-1, 0x0007FFFF
	case 26:
		minOffset, maxOffset, mask = -(1 << 25), (1 << 25)-1, 0x03FFFFFF
	default:
		return 0, fmt.Errorf("unsupported branch offset bit size: %d", bitSize)
	}

	if offset < minOffset || offset > maxOffset {
		return 0, fmt.Errorf("branch offset out of range [%d, %d]: %d instructions", minOffset, maxOffset, offset)
	}

	return uint32(offset) & mask, nil
}

func resolveLabel(target Label, labels map[string]int) (int, error) {
	addr, ok := labels[string(target)]
	if !ok {
		return 0, fmt.Errorf("undefined label: %s", target)
	}
	return addr, nil
}

func calculateByteOffset(targetLabel Label, labels map[string]int, currentPC int) (int64, error) {
	targetAddr, err := resolveLabel(targetLabel, labels)
	if err != nil {
		return 0, err
	}
	return int64(targetAddr*4) - int64(currentPC*4), nil
}

func calculateInstructionOffset(targetLabel Label, labels map[string]int, currentPC int) (int64, error) {
	targetAddr, err := resolveLabel(targetLabel, labels)
	if err != nil {
		return 0, err
	}
	return int64(targetAddr) - int64(currentPC), nil
}

func (i *DataProcessingImmediate) MarshalJSON() ([]byte, error) {
	type Alias DataProcessingImmediate
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *DataProcessingImmediate) UnmarshalJSON(data []byte) error {
	type Alias DataProcessingImmediate
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *DataProcessingImmediate) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	li := LogicalInstruction{
		Opcode:   i.IType,
		Rd:       &i.DestReg,
		Rn:       &i.SourceReg,
		ImmKind:  ImmKindArithmetic,
		ImmValue: i.Imm.Value,
		ImmShift: i.Imm.Shift,
		SetFlags: i.SetFlags,
	}
	if i.DestReg.Is64 != i.SourceReg.Is64 {
		return li, errors.New("source and destination register sizes must match")
	}
	return li, nil
}

func (i *MoveWideImmediate) MarshalJSON() ([]byte, error) {
	type Alias MoveWideImmediate
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *MoveWideImmediate) UnmarshalJSON(data []byte) error {
	type Alias MoveWideImmediate
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *MoveWideImmediate) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	li := LogicalInstruction{
		Opcode:   i.IType,
		Rd:       &i.DestReg,
		ImmKind:  ImmKindMoveWide,
		ImmValue: int64(i.Imm), // Store uint16 as int64
		ImmShift: i.HW,       // Use ImmShift for hw
	}
	return li, nil
}

func (i *PCRelativeAddressing) MarshalJSON() ([]byte, error) {
	type Alias PCRelativeAddressing
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *PCRelativeAddressing) UnmarshalJSON(data []byte) error {
	type Alias PCRelativeAddressing
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *PCRelativeAddressing) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	offset, err := calculateByteOffset(i.Target, labels, currentPC)
	if err != nil {
		return LogicalInstruction{}, err
	}
	li := LogicalInstruction{
		Opcode:   i.IType,
		Rd:       &i.DestReg,
		ImmKind:  ImmKindOffset,
		ImmValue: offset,
	}
	return li, nil
}

func (i *LoadStoreImmediate) MarshalJSON() ([]byte, error) {
	type Alias LoadStoreImmediate
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *LoadStoreImmediate) UnmarshalJSON(data []byte) error {
	type Alias LoadStoreImmediate
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *LoadStoreImmediate) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	li := LogicalInstruction{
		Opcode:   i.IType,
		Rt:       &i.DataReg, // Rt holds the data register
		Rn:       &i.BaseReg,
		ImmKind:  ImmKindOffset,
		ImmValue: i.Offset.Value, // Store unscaled byte offset
	}
	if i.DataReg.Is64 {
		li.MemSize = 64
	} else {
		li.MemSize = 32
	}
	if i.IsLoad {
		li.MemOp = MemOpLoad
	} else {
		li.MemOp = MemOpStore
	}
	return li, nil
}

func (i *UnconditionalBranchImmediate) MarshalJSON() ([]byte, error) {
	type Alias UnconditionalBranchImmediate
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *UnconditionalBranchImmediate) UnmarshalJSON(data []byte) error {
	type Alias UnconditionalBranchImmediate
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *UnconditionalBranchImmediate) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	offset, err := calculateInstructionOffset(i.Target, labels, currentPC)
	if err != nil {
		return LogicalInstruction{}, err
	}
	li := LogicalInstruction{
		Opcode:   i.IType,
		ImmKind:  ImmKindBranch,
		ImmValue: offset,
		IsLink:   i.IsLink,
	}
	return li, nil
}

func (i *ConditionalBranchImmediate) MarshalJSON() ([]byte, error) {
	type Alias ConditionalBranchImmediate
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *ConditionalBranchImmediate) UnmarshalJSON(data []byte) error {
	type Alias ConditionalBranchImmediate
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *ConditionalBranchImmediate) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	offset, err := calculateInstructionOffset(i.Target, labels, currentPC)
	if err != nil {
		return LogicalInstruction{}, err
	}
	li := LogicalInstruction{
		Opcode:    i.IType,
		ImmKind:   ImmKindBranch,
		ImmValue:  offset,
		Condition: i.Condition,
	}
	return li, nil
}

func (i *UnconditionalBranchRegister) MarshalJSON() ([]byte, error) {
	type Alias UnconditionalBranchRegister
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *UnconditionalBranchRegister) UnmarshalJSON(data []byte) error {
	type Alias UnconditionalBranchRegister
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *UnconditionalBranchRegister) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	li := LogicalInstruction{
		Opcode: i.IType,
		Rn:     &i.TargetReg, // Target register for RET/BR goes in Rn field
	}
	return li, nil
}

func (i *NoOperandInstruction) MarshalJSON() ([]byte, error) {
	type Alias NoOperandInstruction
	return json.Marshal(&struct { *Alias }{ Alias: (*Alias)(i) })
}
func (i *NoOperandInstruction) UnmarshalJSON(data []byte) error {
	type Alias NoOperandInstruction
	aux := &struct { *Alias }{ Alias: (*Alias)(i) }
	if err := json.Unmarshal(data, &aux); err != nil { return err }
	i.IType = aux.IType; return nil
}
func (i *NoOperandInstruction) EncodeLogical(labels map[string]int, currentPC int) (LogicalInstruction, error) {
	li := LogicalInstruction{
		Opcode: i.IType,
	}
	return li, nil
}

func NewADDImm(dest, src Register, imm int64) *DataProcessingImmediate {
	return &DataProcessingImmediate{
		BaseInstruction: BaseInstruction{IType: ADDImm},
		DestReg:         dest, SourceReg: src, Imm: Immediate{Value: imm}, SetFlags: false,
	}
}
func NewSUBImm(dest, src Register, imm int64) *DataProcessingImmediate {
	return &DataProcessingImmediate{
		BaseInstruction: BaseInstruction{IType: SUBImm},
		DestReg:         dest, SourceReg: src, Imm: Immediate{Value: imm}, SetFlags: false,
	}
}
func NewMOVZ(dest Register, imm uint16, hw uint) *MoveWideImmediate {
	return &MoveWideImmediate{
		BaseInstruction: BaseInstruction{IType: MOVZ},
		DestReg:         dest, Imm: imm, HW: hw,
	}
}
func NewADR(dest Register, target Label) *PCRelativeAddressing {
	return &PCRelativeAddressing{
		BaseInstruction: BaseInstruction{IType: ADR},
		DestReg:         dest, Target: target,
	}
}
func NewLDRImm(data, base Register, offset int64) *LoadStoreImmediate {
	return &LoadStoreImmediate{
		BaseInstruction: BaseInstruction{IType: LDRImm},
		DataReg:         data, BaseReg: base, Offset: Immediate{Value: offset}, IsLoad: true,
	}
}
func NewSTRImm(data, base Register, offset int64) *LoadStoreImmediate {
	return &LoadStoreImmediate{
		BaseInstruction: BaseInstruction{IType: STRImm},
		DataReg:         data, BaseReg: base, Offset: Immediate{Value: offset}, IsLoad: false,
	}
}
func NewB(target Label) *UnconditionalBranchImmediate {
	return &UnconditionalBranchImmediate{
		BaseInstruction: BaseInstruction{IType: B},
		Target:          target, IsLink: false,
	}
}
func NewBL(target Label) *UnconditionalBranchImmediate {
	return &UnconditionalBranchImmediate{
		BaseInstruction: BaseInstruction{IType: BL},
		Target:          target, IsLink: true,
	}
}
func NewBCond(cond ConditionCode, target Label) *ConditionalBranchImmediate {
	return &ConditionalBranchImmediate{
		BaseInstruction: BaseInstruction{IType: BCond},
		Target:          target, Condition: cond,
	}
}
func NewRET(targetReg Register) *UnconditionalBranchRegister {
	return &UnconditionalBranchRegister{
		BaseInstruction: BaseInstruction{IType: RET},
		TargetReg:       targetReg,
	}
}
func NewNOP() *NoOperandInstruction {
	return &NoOperandInstruction{
		BaseInstruction: BaseInstruction{IType: NOP},
	}
}

func encodeLogicalInstruction(li LogicalInstruction) (uint32, error) {
	switch li.Opcode {
	case ADDImm, SUBImm:
		if li.Rd == nil || li.Rn == nil || li.ImmKind != ImmKindArithmetic {
			return 0, fmt.Errorf("invalid operands for %v", li.Opcode)
		}
		opSize, err := getRegSize(li.Rd)
		if err != nil { return 0, err }
		rnSize, err := getRegSize(li.Rn)
		if err != nil { return 0, err }
		if opSize != rnSize { return 0, errors.New("register size mismatch") }

		sf := uint32(0)
		if opSize == 64 { sf = 1 << 31 }
		opc := uint32(0)
		if li.Opcode == SUBImm { opc = 0b01 << 29 } // ADD is 0b00
		if li.SetFlags {
			if li.Rd.Num == 31 { return 0, errors.New("cannot set flags with ZR/SP destination") }
			opc |= (1 << 29) // S bit
		}
		opi := uint32(0b010 << 23) // Fixed bits for ADDS/SUBS immediate

		imm12Enc, err := encodeImm12ShiftForLogical(li)
		if err != nil { return 0, err }
		rnEnc, _ := encodeReg(li.Rn)
		rdEnc, _ := encodeReg(li.Rd)

		return sf | opc | opi | imm12Enc | (rnEnc << 5) | rdEnc, nil

	case MOVZ:
		if li.Rd == nil || li.ImmKind != ImmKindMoveWide {
			return 0, fmt.Errorf("invalid operands for %v", li.Opcode)
		}
		opSize, err := getRegSize(li.Rd)
		if err != nil { return 0, err }
		if opSize != 64 { return 0, errors.New("MOVZ requires a 64-bit destination register") }
		hw := li.ImmShift
		if hw > 3 { return 0, fmt.Errorf("invalid hardware shift value (0-3): %d", hw) }
		if li.ImmValue < 0 || li.ImmValue > 0xFFFF { return 0, fmt.Errorf("MOVZ immediate out of range (0-65535): %d", li.ImmValue) }

		sf := uint32(1 << 31)
		opc := uint32(0b10 << 29)
		opi := uint32(0b101 << 23)
		hwEnc := uint32(hw) << 21
		imm16Enc := uint32(li.ImmValue) << 5
		rdEnc, _ := encodeReg(li.Rd)

		return sf | opc | opi | hwEnc | imm16Enc | rdEnc, nil

	case ADR:
		if li.Rd == nil || li.ImmKind != ImmKindOffset {
			return 0, fmt.Errorf("invalid operands for %v", li.Opcode)
		}
		offsetEnc, err := encodePCRelOffsetForLogical(li)
		if err != nil { return 0, err }
		op := uint32(0) // op=0 for ADR
		rdEnc, _ := encodeReg(li.Rd)
		return op | uint32(0b10000<<24) | offsetEnc | rdEnc, nil

	case LDRImm, STRImm:
		if li.Rt == nil || li.Rn == nil || li.ImmKind != ImmKindOffset {
			return 0, fmt.Errorf("invalid operands for %v", li.Opcode)
		}
		opSize, err := getRegSize(li.Rt)
		if err != nil { return 0, err }
		baseSize, err := getRegSize(li.Rn)
		if err != nil { return 0, err }
		if !baseSize.Is64 { return 0, errors.New("load/store base register must be 64-bit (Xn or SP)") }

		sizeBits := uint32(0)
		if opSize == 64 { sizeBits = 0b11 } else { sizeBits = 0b10 }

		offsetEnc, err := encodeLoadStoreImmOffsetForLogical(li)
		if err != nil { return 0, err }
		opc := uint32(0)
		if li.MemOp == MemOpLoad { opc = 0b01 } // LDR is 01, STR is 00

		fixedBits := uint32(0b111001 << 24) // Base for LDR/STR immediate unsigned offset
		sizeField := sizeBits << 30
		opcField := opc << 22
		rnEnc, _ := encodeReg(li.Rn)
		rtEnc, _ := encodeReg(li.Rt)

		return sizeField | fixedBits | opcField | offsetEnc | (rnEnc << 5) | rtEnc, nil

	case B, BL:
		if li.ImmKind != ImmKindBranch {
			return 0, fmt.Errorf("invalid operands for %v", li.Opcode)
		}
		imm26Enc, err := encodeBranchOffsetForLogical(li, 26)
		if err != nil { return 0, err }
		op := uint32(0)
		if li.IsLink { op = 1 }
		return (op << 31) | (uint32(0b00101) << 26) | imm26Enc, nil

	case BCond:
		if li.ImmKind != ImmKindBranch {
			return 0, fmt.Errorf("invalid operands for %v", li.Opcode)
		}
		imm19Enc, err := encodeBranchOffsetForLogical(li, 19)
		if err != nil { return 0, err }
		fixedBits := uint32(0b01010100 << 24)
		imm19Field := imm19Enc << 5
		cond := uint32(li.Condition & 0xF)
		return fixedBits | imm19Field | cond, nil

	case RET:
		if li.Rn == nil { // RET uses Rn field for the link register operand
			return 0, fmt.Errorf("missing target register for %v", li.Opcode)
		}
		rnEnc, _ := encodeReg(li.Rn)
		// RET encoding: 1101011 0 0101 11110 000 10 00 Rn 00000
		return uint32(0b11010110010111110000001000000000) | (rnEnc << 5), nil

	case NOP:
		return 0xD503201F, nil

	default:
		return 0, fmt.Errorf("opcode %v not implemented in encodeLogicalInstruction", li.Opcode)
	}
}

func Assemble(program *Program) ([]byte, error) {
	var machineCode []byte
	resolvedLabels := make(map[string]int)

	if program.Labels != nil {
		for name, index := range program.Labels {
			if index < 0 || index > len(program.Instructions) {
				return nil, fmt.Errorf("label '%s' points outside instruction bounds: index %d, length %d", name, index, len(program.Instructions))
			}
			resolvedLabels[name] = index
		}
	}

	for i, node := range program.Instructions {
		if node == nil {
			return nil, fmt.Errorf("nil instruction at index %d", i)
		}
		logicalInstr, err := node.EncodeLogical(resolvedLabels, i)
		if err != nil {
			return nil, fmt.Errorf("error converting node %d (%T, type %v) to logical instruction: %w", i, node, node.InstructionType(), err)
		}

		instrEncoding, err := encodeLogicalInstruction(logicalInstr)
		if err != nil {
			return nil, fmt.Errorf("error encoding logical instruction %d (%v): %w", i, logicalInstr.Opcode, err)
		}

		buf := make([]byte, 4)
		binary.LittleEndian.PutUint32(buf, instrEncoding)
		machineCode = append(machineCode, buf...)
	}

	return machineCode, nil
}

func MarshalNode(node Node) ([]byte, error) {
	return json.Marshal(node)
}

func UnmarshalNode(data []byte) (Node, error) {
	var base struct { Type InstructionType `json:"type"` }
	if err := json.Unmarshal(data, &base); err != nil { return nil, fmt.Errorf("error unmarshaling node type: %w", err) }

	var node Node
	switch base.Type {
	case ADDImm, SUBImm: node = &DataProcessingImmediate{}
	case MOVZ: node = &MoveWideImmediate{}
	case ADR: node = &PCRelativeAddressing{}
	case LDRImm, STRImm: node = &LoadStoreImmediate{}
	case B, BL: node = &UnconditionalBranchImmediate{}
	case BCond: node = &ConditionalBranchImmediate{}
	case RET: node = &UnconditionalBranchRegister{}
	case NOP: node = &NoOperandInstruction{}
	default: return nil, fmt.Errorf("unrecognized instruction type: %v", base.Type)
	}

	if err := json.Unmarshal(data, node); err != nil { return nil, fmt.Errorf("error unmarshaling node data for type %v: %w", base.Type, err) }
	return node, nil
}

func (p *Program) MarshalJSON() ([]byte, error) {
	aux := &struct { Instructions []json.RawMessage `json:"instructions"`; Labels map[string]int `json:"labels"` }{ Labels: p.Labels }
	if p.Instructions == nil { aux.Instructions = []json.RawMessage{} } else {
		aux.Instructions = make([]json.RawMessage, len(p.Instructions))
		for i, node := range p.Instructions {
			data, err := MarshalNode(node)
			if err != nil { return nil, fmt.Errorf("failed to marshal instruction %d: %w", i, err) }
			aux.Instructions[i] = data
		}
	}
	return json.Marshal(aux)
}

func (p *Program) UnmarshalJSON(data []byte) error {
	aux := &struct { Instructions []json.RawMessage `json:"instructions"`; Labels map[string]int `json:"labels"` }{}
	if err := json.Unmarshal(data, &aux); err != nil { return fmt.Errorf("failed to unmarshal program structure: %w", err) }
	p.Labels = aux.Labels
	if aux.Instructions == nil { p.Instructions = nil } else {
		p.Instructions = make([]Node, len(aux.Instructions))
		for i, raw := range aux.Instructions {
			if len(raw) == 0 || string(raw) == "null" { p.Instructions[i] = nil; continue }
			node, err := UnmarshalNode(raw)
			if err != nil { return fmt.Errorf("failed to unmarshal instruction %d: %w", i, err) }
			p.Instructions[i] = node
		}
	}
	return nil
}

func RegW(num int) Register {
	if num < 0 || num > 31 { panic(fmt.Sprintf("invalid W register number: %d", num)) }
	return Register{Num: num, Is64: false}
}
func RegX(num int) Register {
	if num < 0 || num > 31 { panic(fmt.Sprintf("invalid X register number: %d", num)) }
	return Register{Num: num, Is64: true}
}
var WZR = RegW(31)
var XZR = RegX(31)
var SP = RegX(31)
var WSP = RegW(31)
var LR = RegX(30)
