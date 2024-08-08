package tree_sitter_sti_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-sti"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_sti.Language())
	if language == nil {
		t.Errorf("Error loading Sti grammar")
	}
}
