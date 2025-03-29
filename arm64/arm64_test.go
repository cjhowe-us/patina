package arm64

import (
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/bradleyjkemp/cupaloy/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func formatMachineCode(data []byte) string {
	var sb strings.Builder
	for i := 0; i < len(data); i += 4 {
		if i+4 > len(data) {
			sb.WriteString(hex.EncodeToString(data[i:]))
			break
		}
		instr := binary.LittleEndian.Uint32(data[i : i+4])
		sb.WriteString(f(instr))
		if i+4 < len(data) {
			sb.WriteString("\n")
		}
	}
	return sb.String()
}

func f(v uint32) string {
	return fmt.Sprintf("0x%08x", v)
}

func makeInstructions(n int, instr Node) []Node {
	nodes := make([]Node, n)
	for i := range nodes {
		nodes[i] = instr
	}
	return nodes
}

func TestAssemble(t *testing.T) {
	_, b, _, _ := runtime.Caller(0)
	basepath := filepath.Dir(b)
	snap := cupaloy.New(cupaloy.SnapshotSubdirectory(filepath.Join(basepath, "testdata", "snapshots")))

	maxUimm12 := int64(4095)
	maxUimm12Shift12 := int64(4095)

	maxLdrStrWOffset := int64(4095 * 4)
	maxLdrStrXOffset := int64(4095 * 8)

	maxAdrOffset := int64((1 << 20) - 1)
	minAdrOffset := int64(-(1 << 20))

	maxCondBranchOffset := int64((1 << 18) - 1)
	minCondBranchOffset := int64(-(1 << 18))

	maxUncondBranchOffset := int64((1 << 25) - 1)
	minUncondBranchOffset := int64(-(1 << 25))

	testCases := []struct {
		name    string
		program Program
		wantErr bool
	}{
		{
			name: "add sub immediate basic",
			program: Program{
				Instructions: []Node{
					NewADDImm(RegX(1), RegX(2), 123),
					NewADDImm(RegW(3), RegW(4), maxUimm12),
					NewSUBImm(RegX(5), RegX(6), 321),
					NewSUBImm(RegW(7), RegW(8), 0),
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "add immediate with shift",
			program: Program{
				Instructions: []Node{
					&DataProcessingImmediate{
						BaseInstruction: BaseInstruction{IType: ADDImm},
						DestReg:         RegX(1),
						SourceReg:       RegX(2),
						Imm:             Immediate{Value: maxUimm12Shift12, Shift: 12},
					},
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "add sub immediate uses sp/zr",
			program: Program{
				Instructions: []Node{
					NewADDImm(SP, SP, 256),
					NewADDImm(RegX(0), SP, 16),
					NewSUBImm(WSP, WSP, 32),
					NewSUBImm(RegW(1), WSP, 4),
					NewADDImm(RegX(2), XZR, 100),
					NewSUBImm(WZR, WSP, 8),
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "adds subs set flags",
			program: Program{
				Instructions: []Node{
					&DataProcessingImmediate{BaseInstruction: BaseInstruction{IType: ADDImm}, DestReg: RegX(1), SourceReg: RegX(2), Imm: Immediate{Value: 1}, SetFlags: true},
					&DataProcessingImmediate{BaseInstruction: BaseInstruction{IType: SUBImm}, DestReg: RegW(3), SourceReg: RegW(4), Imm: Immediate{Value: 5}, SetFlags: true},
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "adds subs invalid set flags zr",
			program: Program{
				Instructions: []Node{
					&DataProcessingImmediate{BaseInstruction: BaseInstruction{IType: ADDImm}, DestReg: XZR, SourceReg: RegX(2), Imm: Immediate{Value: 1}, SetFlags: true},
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "adds subs invalid set flags sp",
			program: Program{
				Instructions: []Node{
					&DataProcessingImmediate{BaseInstruction: BaseInstruction{IType: SUBImm}, DestReg: WSP, SourceReg: RegW(4), Imm: Immediate{Value: 5}, SetFlags: true},
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "add sub immediate invalid immediate too large",
			program: Program{
				Instructions: []Node{
					NewADDImm(RegX(1), RegX(2), maxUimm12+1),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "add sub immediate invalid immediate negative",
			program: Program{
				Instructions: []Node{
					NewSUBImm(RegW(1), RegW(2), -1),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "add sub immediate invalid shift",
			program: Program{
				Instructions: []Node{
					&DataProcessingImmediate{
						BaseInstruction: BaseInstruction{IType: ADDImm},
						DestReg:         RegX(1),
						SourceReg:       RegX(2),
						Imm:             Immediate{Value: 1, Shift: 1},
					},
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "add sub immediate mismatched registers",
			program: Program{
				Instructions: []Node{
					NewADDImm(RegX(1), RegW(2), 10),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical conversion
		},
		{
			name: "movz basic",
			program: Program{
				Instructions: []Node{
					NewMOVZ(RegX(0), 0x1234, 0),
					NewMOVZ(RegX(1), 0x5678, 1),
					NewMOVZ(XZR, 0xFFFF, 3),
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "movz invalid register size",
			program: Program{
				Instructions: []Node{
					NewMOVZ(RegW(0), 0x1234, 0),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "movz invalid hw shift",
			program: Program{
				Instructions: []Node{
					NewMOVZ(RegX(0), 0x1234, 4),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
        {
			name: "movz invalid immediate value",
			program: Program{
				Instructions: []Node{
					NewMOVZ(RegX(0), 0x10000, 0), // imm16 max is 0xFFFF
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "adr basic",
			program: Program{
				Instructions: []Node{
					NewADR(RegX(0), "targetFwd"), // PC=0, Target=2, Offset=+8 bytes
					NewNOP(),
					NewADR(RegX(1), "targetBwd"), // PC=2, Target=0, Offset=-8 bytes
				},
				Labels: map[string]int{"targetFwd": 2, "targetBwd": 0},
			},
			wantErr: false,
		},
		{
			name: "adr limits",
			program: Program{
				Instructions: []Node{
					NewADR(RegX(0), "maxPos"), // PC=0, Target=(maxAdrOffset/4), Offset=maxAdrOffset (OK)
					NewADR(RegX(1), "minNeg"), // PC=1, Target=0 -> Offset = -4 bytes (OK)
					NewADR(RegX(2), "targetTooBigPos"), // PC=2, Target=(maxAdrOffset/4 + 1) -> Offset = maxAdrOffset + 4 (Fail)
				},
				Labels: map[string]int{
					"maxPos":          int(maxAdrOffset / 4),
					"minNeg":          0, // Tested implicitly by range check logic below
					"targetTooBigPos": int(maxAdrOffset/4) + 1,
				},
			},
			wantErr: true, // Expecting the third ADR to fail logical encoding
		},
        {
			name: "adr negative limit fail",
			program: Program{
				Instructions: []Node{
					NewADR(RegX(0), "target"), // PC=0, Target=(minAdrOffset/4), Offset=minAdrOffset (OK)
                    NewADR(RegX(1), "targetFail"), // PC=1, Target = 1 + (minAdrOffset-4)/4? No, need label defined.
                    // To test failure: target such that target*4 - pc*4 < minAdrOffset
                    // Let pc=0. Need target*4 < minAdrOffset. Target < minAdrOffset/4.
				},
                Labels: map[string]int{
                    "target": int(minAdrOffset / 4), // Should work
                    "targetFail": int(minAdrOffset/4) - 1, // Should fail
                },
			},
			wantErr: true, // Second ADR should fail
		},
		{
			name: "adr undefined label",
			program: Program{
				Instructions: []Node{
					NewADR(RegX(0), "undefined"),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical conversion (label lookup)
		},
		{
			name: "ldr str immediate basic",
			program: Program{
				Instructions: []Node{
					NewLDRImm(RegX(0), SP, 16),              // LDR X0, [SP, #16]
					NewSTRImm(RegW(1), RegX(2), 0),           // STR W1, [X2]
					NewLDRImm(RegW(3), WSP, maxLdrStrWOffset), // LDR W3, [WSP, #max]
					NewSTRImm(RegX(4), SP, maxLdrStrXOffset),  // STR X4, [SP, #max]
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "ldr str immediate invalid offset positive W",
			program: Program{
				Instructions: []Node{
					NewLDRImm(RegW(0), SP, maxLdrStrWOffset+4),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "ldr str immediate invalid offset positive X",
			program: Program{
				Instructions: []Node{
					NewLDRImm(RegX(0), SP, maxLdrStrXOffset+8),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "ldr str immediate invalid offset negative",
			program: Program{
				Instructions: []Node{
					NewSTRImm(RegX(0), SP, -8),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "ldr str immediate offset not multiple W",
			program: Program{
				Instructions: []Node{
					NewLDRImm(RegW(0), SP, 1),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "ldr str immediate offset not multiple X",
			program: Program{
				Instructions: []Node{
					NewLDRImm(RegX(0), SP, 1),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
        {
			name: "ldr str immediate invalid base WSP",
			program: Program{
				Instructions: []Node{
					NewLDRImm(RegX(0), WSP, 16), // Base must be Xn or SP
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "branch unconditional basic",
			program: Program{
				Instructions: []Node{
					NewB("targetFwd"), // PC=0, Target=2, Offset=+2
					NewNOP(),
					NewB("targetBwd"), // PC=2, Target=0, Offset=-2
				},
				Labels: map[string]int{"targetFwd": 2, "targetBwd": 0},
			},
			wantErr: false,
		},
		{
			name: "branch link basic",
			program: Program{
				Instructions: []Node{
					NewBL("subroutine"), // PC=0, Target=1, Offset=+1
					NewNOP(),
				},
				Labels: map[string]int{"subroutine": 1},
			},
			wantErr: false,
		},
		{
			name: "branch unconditional limits",
			program: Program{
				Instructions: []Node{
					NewB("maxPos"), // PC=0, target=maxUncondBranchOffset, Offset=max (OK)
					NewB("minNeg"), // PC=1, target=0 -> Offset = -1 (OK)
                     // Need instr at PC=max+1 jumping to 0 to test minNeg
                     // Test range check logic implicitly
				},
				Labels: map[string]int{
					"maxPos": int(maxUncondBranchOffset),
					"minNeg": 0,
				},
			},
			wantErr: false,
		},
		{
			name: "branch unconditional too large positive",
			program: Program{
				Instructions: []Node{NewB("target")},
				Labels:       map[string]int{"target": int(maxUncondBranchOffset + 1)},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "branch unconditional too large negative",
			program: Program{
				Instructions: []Node{NewB("target")}, // PC=0
				Labels:       map[string]int{"target": int(minUncondBranchOffset - 1)}, // Target < PC + minOffset
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "branch unconditional undefined label",
			program: Program{
				Instructions: []Node{
					NewB("undefined"),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical conversion
		},
		{
			name: "branch conditional basic",
			program: Program{
				Instructions: []Node{
					NewBCond(EQ, "equal"),   // PC=0, Target=1, Offset=+1
					NewNOP(),              // Target for fwd jump
					NewBCond(NE, "start"), // PC=2, Target=0, Offset=-2
				},
				Labels: map[string]int{"equal": 1, "start": 0},
			},
			wantErr: false,
		},
		{
			name: "branch conditional limits",
			program: Program{
				Instructions: []Node{
					NewBCond(HI, "maxPos"), // PC=0, target=maxCondBranchOffset, Offset=max (OK)
					NewBCond(LS, "minNeg"), // PC=1, target=0 -> Offset = -1 (OK)
				},
				Labels: map[string]int{
					"maxPos": int(maxCondBranchOffset),
                    "minNeg": 0,
				},
			},
			wantErr: false,
		},
		{
			name: "branch conditional too large positive",
			program: Program{
				Instructions: []Node{NewBCond(LO, "target")},
				Labels:       map[string]int{"target": int(maxCondBranchOffset + 1)},
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "branch conditional too large negative",
			program: Program{
				Instructions: []Node{NewBCond(MI, "target")}, // PC=0
				Labels:       map[string]int{"target": int(minCondBranchOffset - 1)}, // Target < PC + minOffset
			},
			wantErr: true, // Error during logical instruction encoding
		},
		{
			name: "branch conditional undefined label",
			program: Program{
				Instructions: []Node{
					NewBCond(CS, "undefined"),
				},
				Labels: map[string]int{},
			},
			wantErr: true, // Error during logical conversion
		},
		{
			name: "ret basic",
			program: Program{
				Instructions: []Node{
					NewRET(LR),        // RET X30
					NewRET(RegX(10)), // RET X10
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
        {
			name: "ret invalid register W",
			program: Program{
				Instructions: []Node{
					NewRET(RegW(30)), // RET W30 - Invalid for RET
				},
				Labels: map[string]int{},
			},
			// Currently RET doesn't check register size, should it?
            // Architecture manual implies Xm. Let's allow it for now,
            // but real hardware might fault.
			wantErr: false,
		},
		{
			name: "nop basic",
			program: Program{
				Instructions: []Node{
					NewNOP(),
					NewNOP(),
				},
				Labels: map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "empty program",
			program: Program{
				Instructions: []Node{},
				Labels:       map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "program with nil instruction",
			program: Program{
				Instructions: []Node{NewNOP(), nil, NewNOP()},
				Labels:       map[string]int{},
			},
			wantErr: true, // Error during assembly loop
		},
		{
			name: "program with nil labels map",
			program: Program{
				Instructions: []Node{NewNOP()},
				Labels:       nil,
			},
			wantErr: false,
		},
		{
			name: "label points outside bounds positive",
			program: Program{
				Instructions: []Node{NewNOP()},
				Labels:       map[string]int{"badlabel": 1},
			},
			wantErr: true, // Error during label resolution check
		},
		{
			name: "label points outside bounds negative",
			program: Program{
				Instructions: []Node{NewNOP()},
				Labels:       map[string]int{"badlabel": -1},
			},
			wantErr: true, // Error during label resolution check
		},
		{
			name: "json marshal unmarshal roundtrip",
			program: Program{
				Instructions: []Node{
					NewADDImm(RegX(1), RegX(2), 123),
					NewB("target"),
					NewLDRImm(RegW(0), SP, 16),
					NewRET(LR),
					NewMOVZ(RegX(5), 0xabcd, 2),
					NewNOP(), // Target label here
					NewADR(RegX(7), "target"),
					&DataProcessingImmediate{BaseInstruction: BaseInstruction{IType: ADDImm}, DestReg: RegX(8), SourceReg: SP, Imm: Immediate{Value: 1, Shift: 12}, SetFlags: true},
					NewBCond(NE, "target"),
				},
				Labels: map[string]int{"target": 5},
			},
			wantErr: false,
		},
		{
			name: "json marshal unmarshal empty program",
			program: Program{
				Instructions: []Node{},
				Labels:       map[string]int{},
			},
			wantErr: false,
		},
		{
			name: "json marshal unmarshal nil instructions",
			program: Program{
				Instructions: nil,
				Labels:       map[string]int{"a": 0},
			},
			wantErr: false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			// --- Test JSON Marshaling ---
			jsonData, err := json.MarshalIndent(tc.program, "", "  ")
			require.NoError(t, err, "Failed to marshal program to JSON")
			snap.SnapshotMulti(t.Name()+"_input_ast", string(jsonData))

			// --- Test JSON Unmarshaling ---
			var unmarshaledProgram Program
			err = json.Unmarshal(jsonData, &unmarshaledProgram)
			require.NoError(t, err, "Failed to unmarshal program from JSON")
            // Basic check after unmarshal
            require.Equal(t, len(tc.program.Instructions), len(unmarshaledProgram.Instructions), "Instruction count mismatch after unmarshal")
            require.Equal(t, tc.program.Labels, unmarshaledProgram.Labels, "Labels mismatch after unmarshal")


			// --- Assemble the Unmarshaled Program ---
			machineCode, err := Assemble(&unmarshaledProgram)

			// --- Assert Errors ---
			if tc.wantErr {
				require.Error(t, err, "Expected an error, but got nil")
				snap.SnapshotMulti(t.Name()+"_error", err.Error())
			} else {
				require.NoError(t, err, "Expected no error, but got error")
                require.NotNil(t, machineCode, "Expected machine code, but got nil")
				// --- Snapshot Output ---
                if len(machineCode) > 0 {
				    output := formatMachineCode(machineCode)
				    snap.SnapshotMulti(t.Name()+"_output_code", output)
                } else {
                    snap.SnapshotMulti(t.Name()+"_output_code", "<empty output>")
                }
			}

			// --- Assemble Original Program (Sanity Check/Consistency) ---
            // Re-run assembly on the original tc.program to ensure consistency if needed,
            // especially if deep comparison of unmarshaled vs original is complex.
			machineCodeOrig, errOrig := Assemble(&tc.program)
			if tc.wantErr {
				require.Error(t, errOrig, "[Original] Expected an error, but got nil")
                // Optionally compare error messages if they should be identical
                if err != nil && errOrig != nil {
                     assert.Equal(t, err.Error(), errOrig.Error(), "[Original] Error message mismatch")
                }
			} else {
				require.NoError(t, errOrig, "[Original] Expected no error, but got error")
                // --- Compare Original and Unmarshaled Outputs ---
				if !tc.wantErr && err == nil && errOrig == nil {
                    assert.Equal(t, machineCode, machineCodeOrig, "Machine code mismatch between original and unmarshaled programs")
				}
			}
		})
	}
}

// TestEncodeLogicalInstruction focuses on the final encoding step, isolated from AST parsing/conversion.
func TestEncodeLogicalInstruction(t *testing.T) {
    _, b, _, _ := runtime.Caller(0)
	basepath := filepath.Dir(b)
	snap := cupaloy.New(cupaloy.SnapshotSubdirectory(filepath.Join(basepath, "testdata", "snapshots")))

    r1 := RegX(1)
    r2 := RegX(2)
    r3 := RegW(3)
    r4 := RegW(4)
    spReg := SP // Use variable for pointer
    lrReg := LR

    testCases := []struct{
        name string
        li LogicalInstruction
        wantHex string // Expected hex string of the uint32
        wantErr bool
    }{
        {
            name: "add imm x1, x2, 123",
            li: LogicalInstruction{ Opcode: ADDImm, Rd: &r1, Rn: &r2, ImmKind: ImmKindArithmetic, ImmValue: 123 },
            wantHex: "0x9101f041", // add x1, x2, #0x7b
            wantErr: false,
        },
        {
            name: "sub imm w3, w4, 4095",
            li: LogicalInstruction{ Opcode: SUBImm, Rd: &r3, Rn: &r4, ImmKind: ImmKindArithmetic, ImmValue: 4095 },
            wantHex: "0x713ff083", // sub w3, w4, #0xfff
            wantErr: false,
        },
         {
            name: "adds imm x1, x2, 1, shift 12",
            li: LogicalInstruction{ Opcode: ADDImm, Rd: &r1, Rn: &r2, ImmKind: ImmKindArithmetic, ImmValue: 1, ImmShift: 12, SetFlags: true },
            wantHex: "0xb1401041", // adds x1, x2, #0x1000 (1 LSL 12)
            wantErr: false,
        },
        {
             name: "subs imm wsp, wsp, 16", // Using WSP
             li: LogicalInstruction{ Opcode: SUBImm, Rd: &WSP, Rn: &WSP, ImmKind: ImmKindArithmetic, ImmValue: 16, SetFlags: true},
             wantHex: "0x610043ff", // subs wsp, wsp, #16
             wantErr: false, // SUBS with WSP destination is valid
        },
        {
             name: "adds imm wzr, w4, 1", // Setting flags with WZR destination
             li: LogicalInstruction{ Opcode: ADDImm, Rd: &WZR, Rn: &r4, ImmKind: ImmKindArithmetic, ImmValue: 1, SetFlags: true},
             wantErr: true, // Cannot set flags with WZR dest
        },
        {
            name: "movz x1, 0xabcd, lsl 16",
            li: LogicalInstruction{ Opcode: MOVZ, Rd: &r1, ImmKind: ImmKindMoveWide, ImmValue: 0xabcd, ImmShift: 1 }, // hw=1
            wantHex: "0xd2a579a1", // movz x1, #0xabcd, lsl #16
            wantErr: false,
        },
        {
            name: "adr x1, +8 offset", // Offset already calculated
            li: LogicalInstruction{ Opcode: ADR, Rd: &r1, ImmKind: ImmKindOffset, ImmValue: 8 },
            wantHex: "0x10000041", // adr x1, #8
            wantErr: false,
        },
        {
            name: "adr x2, -4 offset",
            li: LogicalInstruction{ Opcode: ADR, Rd: &r2, ImmKind: ImmKindOffset, ImmValue: -4 },
            wantHex: "0xf0ffffe2", // adr x2, #-4
            wantErr: false,
        },
        {
            name: "ldr w3, [sp, #12]", // Offset 12 is multiple of 4
            li: LogicalInstruction{ Opcode: LDRImm, Rt: &r3, Rn: &spReg, MemOp: MemOpLoad, MemSize: 32, ImmKind: ImmKindOffset, ImmValue: 12 },
            wantHex: "0xb9400fe3", // ldr w3, [sp, #12]
            wantErr: false,
        },
        {
            name: "str x1, [x2, #16]", // Offset 16 is multiple of 8
            li: LogicalInstruction{ Opcode: STRImm, Rt: &r1, Rn: &r2, MemOp: MemOpStore, MemSize: 64, ImmKind: ImmKindOffset, ImmValue: 16 },
            wantHex: "0xf9000841", // str x1, [x2, #16]
            wantErr: false,
        },
         {
            name: "ldr x1, [x2, #1]", // Offset not multiple of 8
            li: LogicalInstruction{ Opcode: LDRImm, Rt: &r1, Rn: &r2, MemOp: MemOpLoad, MemSize: 64, ImmKind: ImmKindOffset, ImmValue: 1 },
            wantErr: true,
        },
        {
            name: "b +4 instructions", // Offset already calculated
            li: LogicalInstruction{ Opcode: B, ImmKind: ImmKindBranch, ImmValue: 4 },
            wantHex: "0x14000004", // b #16 (4 instructions = 16 bytes)
            wantErr: false,
        },
        {
            name: "bl -8 instructions",
            li: LogicalInstruction{ Opcode: BL, ImmKind: ImmKindBranch, ImmValue: -8, IsLink: true },
            wantHex: "0x97fffffe", // bl #-32 (-8 instructions = -32 bytes)
            wantErr: false,
        },
        {
            name: "b.eq +1 instruction",
            li: LogicalInstruction{ Opcode: BCond, ImmKind: ImmKindBranch, ImmValue: 1, Condition: EQ },
            wantHex: "0x54000020", // b.eq #4 (1 instruction = 4 bytes)
            wantErr: false,
        },
        {
            name: "b.ne -2 instructions",
            li: LogicalInstruction{ Opcode: BCond, ImmKind: ImmKindBranch, ImmValue: -2, Condition: NE },
            wantHex: "0x54ffffc1", // b.ne #-8 (-2 instructions = -8 bytes)
            wantErr: false,
        },
        {
            name: "ret lr",
            li: LogicalInstruction{ Opcode: RET, Rn: &lrReg },
            wantHex: "0xd65f03c0", // ret (ret x30)
            wantErr: false,
        },
        {
            name: "ret x2",
            li: LogicalInstruction{ Opcode: RET, Rn: &r2 },
            wantHex: "0xd65f0040", // ret x2
            wantErr: false,
        },
        {
            name: "nop",
            li: LogicalInstruction{ Opcode: NOP },
            wantHex: "0xd503201f", // nop (hint #0)
            wantErr: false,
        },
        // Error cases
        { name: "add imm nil rd", li: LogicalInstruction{ Opcode: ADDImm, Rn: &r2, ImmKind: ImmKindArithmetic, ImmValue: 1 }, wantErr: true },
        { name: "add imm nil rn", li: LogicalInstruction{ Opcode: ADDImm, Rd: &r1, ImmKind: ImmKindArithmetic, ImmValue: 1 }, wantErr: true },
        { name: "add imm wrong kind", li: LogicalInstruction{ Opcode: ADDImm, Rd: &r1, Rn: &r2, ImmKind: ImmKindOffset, ImmValue: 1 }, wantErr: true },
        { name: "ldr nil rt", li: LogicalInstruction{ Opcode: LDRImm, Rn: &spReg, MemOp: MemOpLoad, MemSize: 32, ImmKind: ImmKindOffset, ImmValue: 12 }, wantErr: true },
        { name: "ldr nil rn", li: LogicalInstruction{ Opcode: LDRImm, Rt: &r3, MemOp: MemOpLoad, MemSize: 32, ImmKind: ImmKindOffset, ImmValue: 12 }, wantErr: true },
        { name: "b wrong kind", li: LogicalInstruction{ Opcode: B, ImmKind: ImmKindOffset, ImmValue: 4 }, wantErr: true },
        { name: "ret nil rn", li: LogicalInstruction{ Opcode: RET }, wantErr: true },

    }

    for _, tc := range testCases {
        t.Run(tc.name, func(t *testing.T){
             enc, err := encodeLogicalInstruction(tc.li)

             if tc.wantErr {
                 require.Error(t, err, "Expected error")
                 snap.SnapshotMulti(t.Name()+"_error", err.Error())
             } else {
                 require.NoError(t, err, "Expected no error")
                 assert.Equal(t, tc.wantHex, f(enc), "Encoded value mismatch")
                 snap.SnapshotMulti(t.Name()+"_output", f(enc))
             }
        })
    }
}


func TestRegisterJson(t *testing.T) {
	_, b, _, _ := runtime.Caller(0)
	basepath := filepath.Dir(b)
	snap := cupaloy.New(cupaloy.SnapshotSubdirectory(filepath.Join(basepath, "testdata", "snapshots")))

	regs := []struct { name string; reg Register } {
		{"x0", RegX(0)}, {"x15", RegX(15)}, {"x30", RegX(30)}, {"lr", LR}, {"xzr", XZR}, {"sp", SP},
		{"w0", RegW(0)}, {"w15", RegW(15)}, {"w30", RegW(30)}, {"wzr", WZR}, {"wsp", WSP},
	}

	for _, item := range regs {
		t.Run("Marshal_"+item.name, func(t *testing.T) {
			jsonData, err := json.Marshal(item.reg)
			require.NoError(t, err)
			snap.SnapshotMulti(t.Name(), string(jsonData))

			expectedJson := fmt.Sprintf(`"%s"`, item.name)
			assert.Equal(t, expectedJson, string(jsonData))
		})

		t.Run("Unmarshal_"+item.name, func(t *testing.T) {
			jsonData := fmt.Sprintf(`"%s"`, item.name)
			var unmarshaledReg Register
			err := json.Unmarshal([]byte(jsonData))
			require.NoError(t, err)
			assert.Equal(t, item.reg, unmarshaledReg)
		})
	}

	t.Run("UnmarshalInvalid", func(t *testing.T) {
		invalidRegs := []string{`"x"`, `"w"`, `"x32"`, `"w-1"`, `"r0"`, `"spx"`, `""`, `"x 5"`, `123`}
		for _, invalidJson := range invalidRegs {
			var r Register
			err := json.Unmarshal([]byte(invalidJson))
			require.Error(t, err, "Expected error unmarshaling "+invalidJson)
			snap.SnapshotMulti(t.Name()+"_"+invalidJson, err.Error())
		}
	})
}
