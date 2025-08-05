package models

//go:generate go run ../generators/mnstr_xp.go

import (
	"context"
	"crypto/sha1"
	"database/sql"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/mnstrapp/mnstrv2server/database"
)

func XPForMnstrLevel(level int) int {
	if level > len(mnstrXPForLevel)-1 {
		return mnstrXPForLevel[len(mnstrXPForLevel)-1]
	}
	return mnstrXPForLevel[level]
}

type Mnstr struct {
	ID                  string    `json:"id"`
	UserID              string    `json:"userId"`
	Name                string    `json:"name"`
	Description         string    `json:"description"`
	QRCode              string    `json:"qrCode"`
	Level               int       `json:"level"`
	Experience          int       `json:"experience"`
	CurrentHealth       int       `json:"currentHealth"`
	MaxHealth           int       `json:"maxHealth"`
	CurrentAttack       int       `json:"currentAttack"`
	MaxAttack           int       `json:"maxAttack"`
	CurrentDefense      int       `json:"currentDefense"`
	MaxDefense          int       `json:"maxDefense"`
	CurrentSpeed        int       `json:"currentSpeed"`
	MaxSpeed            int       `json:"maxSpeed"`
	CurrentIntelligence int       `json:"currentIntelligence"`
	MaxIntelligence     int       `json:"maxIntelligence"`
	CurrentMagic        int       `json:"currentMagic"`
	MaxMagic            int       `json:"maxMagic"`
	CreatedAt           time.Time `json:"-"`
	UpdatedAt           time.Time `json:"-"`
	ArchivedAt          time.Time `json:"-"`
}

type FoundMnstr struct {
	ID                  string         `json:"id"`
	UserID              string         `json:"userId"`
	Name                sql.NullString `json:"name"`
	Description         sql.NullString `json:"description"`
	QRCode              string         `json:"qrCode"`
	Level               sql.NullInt64  `json:"level"`
	Experience          sql.NullInt64  `json:"experience"`
	CurrentHealth       sql.NullInt64  `json:"currentHealth"`
	MaxHealth           sql.NullInt64  `json:"maxHealth"`
	CurrentAttack       sql.NullInt64  `json:"currentAttack"`
	MaxAttack           sql.NullInt64  `json:"maxAttack"`
	CurrentDefense      sql.NullInt64  `json:"currentDefense"`
	MaxDefense          sql.NullInt64  `json:"maxDefense"`
	CurrentSpeed        sql.NullInt64  `json:"currentSpeed"`
	MaxSpeed            sql.NullInt64  `json:"maxSpeed"`
	CurrentIntelligence sql.NullInt64  `json:"currentIntelligence"`
	MaxIntelligence     sql.NullInt64  `json:"maxIntelligence"`
	CurrentMagic        sql.NullInt64  `json:"currentMagic"`
	MaxMagic            sql.NullInt64  `json:"maxMagic"`
	CreatedAt           time.Time      `json:"-"`
	UpdatedAt           time.Time      `json:"-"`
	ArchivedAt          sql.NullTime   `json:"-"`
}

func NewMnstr(qrCode, userId string) *Mnstr {
	return &Mnstr{
		QRCode: qrCode,
		UserID: userId,
	}
}

func NewMnstrFromFoundMnstr(foundMnstr FoundMnstr) *Mnstr {
	return &Mnstr{
		ID:                  foundMnstr.ID,
		UserID:              foundMnstr.UserID,
		Name:                foundMnstr.Name.String,
		Description:         foundMnstr.Description.String,
		QRCode:              foundMnstr.QRCode,
		Level:               int(foundMnstr.Level.Int64),
		Experience:          int(foundMnstr.Experience.Int64),
		CurrentHealth:       int(foundMnstr.CurrentHealth.Int64),
		MaxHealth:           int(foundMnstr.MaxHealth.Int64),
		CurrentAttack:       int(foundMnstr.CurrentAttack.Int64),
		MaxAttack:           int(foundMnstr.MaxAttack.Int64),
		CurrentDefense:      int(foundMnstr.CurrentDefense.Int64),
		MaxDefense:          int(foundMnstr.MaxDefense.Int64),
		CurrentSpeed:        int(foundMnstr.CurrentSpeed.Int64),
		MaxSpeed:            int(foundMnstr.MaxSpeed.Int64),
		CurrentIntelligence: int(foundMnstr.CurrentIntelligence.Int64),
		MaxIntelligence:     int(foundMnstr.MaxIntelligence.Int64),
		CurrentMagic:        int(foundMnstr.CurrentMagic.Int64),
		MaxMagic:            int(foundMnstr.MaxMagic.Int64),
		CreatedAt:           foundMnstr.CreatedAt,
		UpdatedAt:           foundMnstr.UpdatedAt,
		ArchivedAt:          foundMnstr.ArchivedAt.Time,
	}
}

func FindMnstrByQRCodeForUser(qrCode string, userId string) (*Mnstr, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE mnstr_qr_code = $1 AND user_id = $2 LIMIT 1
	`
	rows, err := db.Query(context.Background(), query, qrCode, userId)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var foundMnstr FoundMnstr
	if rows.Next() {
		err = rows.Scan(&foundMnstr.ID, &foundMnstr.UserID, &foundMnstr.Name, &foundMnstr.Description, &foundMnstr.QRCode, &foundMnstr.CreatedAt, &foundMnstr.UpdatedAt, &foundMnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}
	}
	if foundMnstr.ID == "" {
		return nil, errors.New("mnstr not found")
	}

	mnstr := NewMnstrFromFoundMnstr(foundMnstr)

	return mnstr, nil
}

func GetMnstrsByUserID(userId string) ([]*Mnstr, error) {

	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE user_id = $1
		ORDER BY created_at ASC
	`
	rows, err := db.Query(context.Background(), query, userId)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var mnstrs []*Mnstr
	for rows.Next() {
		var mnstr FoundMnstr
		err = rows.Scan(&mnstr.ID, &mnstr.UserID, &mnstr.Name, &mnstr.Description, &mnstr.QRCode, &mnstr.CreatedAt, &mnstr.UpdatedAt, &mnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}

		if mnstr.ID == "" {
			continue
		}

		mnstrs = append(mnstrs, NewMnstrFromFoundMnstr(mnstr))
	}

	return mnstrs, nil
}

func GetMnstrByID(id string, userId string) (*Mnstr, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE id = $1 AND user_id = $2
	`
	rows, err := db.Query(context.Background(), query, id, userId)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var mnstr FoundMnstr
	if rows.Next() {
		err = rows.Scan(&mnstr.ID, &mnstr.UserID, &mnstr.Name, &mnstr.Description, &mnstr.QRCode, &mnstr.CreatedAt, &mnstr.UpdatedAt, &mnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}
	}

	if mnstr.ID == "" {
		return nil, errors.New("mnstr not found")
	}

	return NewMnstrFromFoundMnstr(mnstr), nil
}

func GetMnstrByQRCode(qrCode string) (*Mnstr, error) {
	db, err := database.Connection()
	if err != nil {
		return nil, err
	}
	defer db.Close(context.Background())

	query := `
		SELECT id, user_id, mnstr_name, mnstr_description, mnstr_qr_code, created_at, updated_at, archived_at
		FROM mnstrs
		WHERE mnstr_qr_code = $1
		LIMIT 1
	`
	rows, err := db.Query(context.Background(), query, qrCode)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var mnstr FoundMnstr
	if rows.Next() {
		err = rows.Scan(&mnstr.ID, &mnstr.UserID, &mnstr.Name, &mnstr.Description, &mnstr.QRCode, &mnstr.CreatedAt, &mnstr.UpdatedAt, &mnstr.ArchivedAt)
		if err != nil {
			return nil, err
		}
	}

	if mnstr.ID == "" {
		return nil, errors.New("mnstr not found")
	}

	return NewMnstrFromFoundMnstr(mnstr), nil
}

func (m *Mnstr) Create() error {
	if m.QRCode == "" {
		return errors.New("qrCode is required")
	}
	if m.UserID == "" {
		return errors.New("userID is required")
	}

	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())
	m.ID = uuid.New().String()

	var newMnstr Mnstr

	err = db.QueryRow(context.Background(), `
	INSERT INTO mnstrs (
		id,
		user_id,
		mnstr_name,
		mnstr_description,
		mnstr_qr_code,
		current_level,
		current_experience,
		current_health,
		max_health,
		current_attack,
		max_attack,
		current_defense,
		max_defense,
		current_speed,
		max_speed,
		current_intelligence,
		max_intelligence,
		current_magic,
		max_magic,
		created_at,
		updated_at
	) VALUES (
		$1,
		$2,
		$3,
		$4,
		$5,
		$6,
		$7,
		$8,
		$9,
		$10,
		$11,
		$12,
		$13,
		$14,
		$15,
		$16,
		$17,
		$18,
		$19,
		$20,
		$21
	) RETURNING
	 	id,
		user_id,
		mnstr_name,
		mnstr_description,
		mnstr_qr_code,
		current_level,
		current_experience,
		current_health,
		max_health,
		current_attack,
		max_attack,
		current_defense,
		max_defense,
		current_speed,
		max_speed,
		current_intelligence,
		max_intelligence,
		current_magic,
		max_magic,
		created_at,
		updated_at
	`,
		m.ID,
		m.UserID,
		m.Name,
		m.Description,
		m.QRCode,
		m.Level,
		m.Experience,
		m.CurrentHealth,
		m.MaxHealth,
		m.CurrentAttack,
		m.MaxAttack,
		m.CurrentDefense,
		m.MaxDefense,
		m.CurrentSpeed,
		m.MaxSpeed,
		m.CurrentIntelligence,
		m.MaxIntelligence,
		m.CurrentMagic,
		m.MaxMagic,
		m.CreatedAt,
		m.UpdatedAt).Scan(
		&newMnstr.ID,
		&newMnstr.UserID,
		&newMnstr.Name,
		&newMnstr.Description,
		&newMnstr.QRCode,
		&newMnstr.Level,
		&newMnstr.Experience,
		&newMnstr.CurrentHealth,
		&newMnstr.MaxHealth,
		&newMnstr.CurrentAttack,
		&newMnstr.MaxAttack,
		&newMnstr.CurrentDefense,
		&newMnstr.MaxDefense,
		&newMnstr.CurrentSpeed,
		&newMnstr.MaxSpeed,
		&newMnstr.CurrentIntelligence,
		&newMnstr.MaxIntelligence,
		&newMnstr.CurrentMagic,
		&newMnstr.MaxMagic,
		&newMnstr.CreatedAt,
		&newMnstr.UpdatedAt,
	)
	if err != nil {
		return err
	}
	user, err := FindUserByID(m.UserID)
	if err != nil {
		return err
	}

	err = user.UpdateXP(XPForMnstrLevel(user.ExperienceLevel))
	if err != nil {
		return err
	}

	coins, err := m.Coins()
	if err != nil {
		return err
	}

	return user.AddCoins(coins)
}

func (m *Mnstr) Update() error {
	db, err := database.Connection()
	if err != nil {
		return err
	}
	defer db.Close(context.Background())

	query := `
		UPDATE mnstrs
		SET mnstr_name = $1, mnstr_description = $2, current_level = $3, current_experience = $4, current_health = $5, max_health = $6, current_attack = $7, max_attack = $8, current_defense = $9, max_defense = $10, current_speed = $11, max_speed = $12, current_intelligence = $13, max_intelligence = $14, current_magic = $15, max_magic = $16, 	updated_at = $17
		WHERE id = $18
	`
	_, err = db.Exec(context.Background(), query, m.Name, m.Description, m.Level, m.Experience, m.CurrentHealth, m.MaxHealth, m.CurrentAttack, m.MaxAttack, m.CurrentDefense, m.MaxDefense, m.CurrentSpeed, m.MaxSpeed, m.CurrentIntelligence, m.MaxIntelligence, m.CurrentMagic, m.MaxMagic, m.UpdatedAt, m.ID)
	if err != nil {
		return err
	}

	return nil
}

func (m *Mnstr) Coins() (int64, error) {
	hash := sha1.New()
	hash.Write([]byte(m.QRCode))
	hashBytes := hash.Sum(nil)
	coinsByte := hashBytes[len(hashBytes)/2]
	multiplierByte := hashBytes[len(hashBytes)/2+1]

	coins := int64(coinsByte)
	if coins <= 0 {
		coins = 5
	}
	multiplier := int64(multiplierByte)
	if multiplier <= 0 {
		multiplier = 10
	}

	if multiplier >= 251 {
		coins = (coins * (multiplier / 100)) + 1000
		if coins > 2000 {
			coins = 2000
		}
	} else if multiplier >= 242 {
		coins = (coins * (multiplier / 100)) + 400
		if coins > 750 {
			coins = 750
		}
	} else if multiplier >= 216 {
		coins = (coins * (multiplier / 100)) + 150
		if coins > 400 {
			coins = 400
		}
	} else {
		if multiplier >= 85 {
			coins = coins * int64(multiplier/100)
		}
		if coins > 25 {
			coins = coins / 10
		}
	}

	if coins <= 5 {
		return 5, nil
	}

	return coins, nil
}
