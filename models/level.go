package models

//go:generate go run ../generators/level_xp.go

func XPForLevel(level int) int {
	if level > len(xpForLevel)-1 {
		return xpForLevel[len(xpForLevel)-1]
	}
	return xpForLevel[level]
}
