package ast

import (
	"os"
	"testing"

	"github.com/gkampitakis/go-snaps/snaps"
)

func TestMain(m *testing.M) {
	v := m.Run()

	snaps.Clean(m, snaps.CleanOpts{Sort: true})

	os.Exit(v)
}

var jsonSnapshotConfig = snaps.JSON(snaps.JSONConfig{
	Width:    80,
	Indent:   "  ",
	SortKeys: true,
})
