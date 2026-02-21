# Tenuto 2.1 Preview

This document outlines the normative changes for the Tenuto 2.1 specification. These updates resolve the **LL(k) ambiguity conflicts** identified in Version 2.0.0 by introducing unique compound sigils for data structures and musical blocks.

---

## 2.6 Operators & Punctuators (Updated)

The following symbols are updated to provide unique entry-points for complex data structures. A conformant parser **MUST** tokenize compound sigils (`<[`, `]>`, `@{`) as single atomic units.

| Symbol | Name | Usage |
| --- | --- | --- |
| `{` `}` | **Structural Braces** | Scope definition for high-level blocks (`tenuto`, `measure`, `group`, `macro`). |
| `<[` `]>` | **Voice Brackets** | **[NEW]** Encloses multi-voice polyphonic blocks (`v1: ... | v2: ...`). |
| `@{` `}` | **Map Sigil** | **[NEW]** Encloses Key-Value Data Maps (Metadata and Attributes). |
| `[` `]` | Brackets | Pitch Chord grouping (`[c4 e4 g4]`) or Volta numbering. |
| `:` | Colon | Assignment (Staff/Voice ID) or Duration separator. |
| `.` | Dot | Attribute accessor (`.stacc`). |
| `,` | Comma | List separator within Maps and Arrays. |
| `~` | Tilde | Tie connection between notes. |
| ` | ` | Pipe |
| `$` | Dollar | Macro invocation prefix. |
| `(` `)` | Parentheses | Tuplet grouping or Macro argument lists. |
| `=` | Equals | Definition assignment (`var` or `macro`). |

---

## 3.3 Metadata Scope & Keys (Updated)

To distinguish configuration data from musical logic, all `meta` blocks and data maps **MUST** be initiated with the `@` sigil. This allows recursive descent parsers to differentiate between a `Value::Map` and a structural block without backtracking.

**Syntax:** `meta @{ key: value, ... }`

* **Global Meta:** Establishes the environment for the entire document.
* **Local Meta:** When used inside a `measure`, it creates a local override that inherits from the Global scope.

```tenuto
%% Global Metadata
meta @{
  title: "Tenuto V2.1 Specification",
  tempo: 120,
  time: "4/4"
}

measure 5 {
  %% Local Meta Override
  meta @{ tempo: 140 }
  sax: c4 d e f |
}

```

---

## 10.1 Voice Group Syntax (Updated)

Polyphonic regions within a single staff are enclosed in **Voice Brackets** (`<[` and `]>`). This unique delimiter ensures the parser can immediately distinguish between a music block and a metadata map following a Staff ID, resolving the "Metadata Trap."

**Syntax:**

```tenuto
Staff_ID: <[
  Voice_ID: Events... |
  Voice_ID: Events... |
]>

```

* **Entry Logic:** The `<[` token triggers the Voice Engine.
* **Voice Separation:** Voices are separated by the Pipe (`|`) character.
* **Exit Logic:** The `]>` token closes the polyphonic scope and restores the "Sticky State" of the primary voice (`v1`) to the Staff cursor.

```tenuto
pno: <[
  v1: c4:4 d e f |
  v2: f2:1        |
]>

```

---

## 26. Formal Grammar (EBNF Sections)

The following EBNF rules are updated to reflect the unambiguous sigils.

```ebnf
/* 26.3 Logic & Events */
Logic         ::= Assignment
                | MetaBlock
                | Conditional

/* Assignment: Identifier followed by either a VoiceGroup or a bracketed MultiVoiceBlock */
Assignment    ::= IDENTIFIER ":" (VoiceGroup | MultiVoiceBlock)

/* Multi-Voice Block: Uses unique <[ ]> delimiters */
MultiVoiceBlock ::= "<[" Voice ("|" Voice)* "]" ">"

/* 26.4 Attributes & Data Structures */
MetaBlock     ::= "meta" "@{" KeyValueList "}"

/* Maps now require the @ sigil to differentiate from Structural Blocks */
Map           ::= "@{" KeyValueList "}"

Value         ::= INTEGER | FLOAT | STRING | IDENTIFIER | Array | Map

```

---

## 28. Reference Example (V2.1 "Kitchen Sink")

```tenuto
tenuto "2.1" {
  meta @{ 
    title: "Tenuto V2.1 Reference", 
    tempo: 130 
  }

  def sax "Tenor Sax" attributes=@{ patch: "piano" }

  measure 1 {
    sax: <[
      v1: c4:4 d e f |
      v2: g3:1        |
    ]>
  }
}

```
