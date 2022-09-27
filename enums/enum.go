package enums

type TokenIDs int32

const (
	FUNCTION TokenIDs = iota
	INT
	FLOAT
	BOOL
	STRING
	END
	IF
	IFF
	ELSE
	WHILE
	DO
	AS
	ASSINGMENT
	VAR // type infereance
	LET
	TYPE
	TFUNCTION
	IDENTIFIER
)
