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
	LET  // type infereance
	TINT // type declarations
	TINT8
	TINT16
	TINT32
	TINT64
	TFLOAT32
	TFLOAT64
	TFLOAT
	TSTRING
	TBOOLEAN
	TFUNCTION
	IDENTIFIER
)
