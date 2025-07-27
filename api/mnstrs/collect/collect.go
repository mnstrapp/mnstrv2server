package collect

import (
	"encoding/json"
	"net/http"
	"strings"

	"github.com/mnstrapp/mnstrv2server/models"
)

type CollectRequest struct {
	QRCode              string `json:"qrCode"`
	Name                string `json:"name"`
	CurrentHealth       int    `json:"currentHealth"`
	MaxHealth           int    `json:"maxHealth"`
	CurrentAttack       int    `json:"currentAttack"`
	MaxAttack           int    `json:"maxAttack"`
	CurrentDefense      int    `json:"currentDefense"`
	MaxDefense          int    `json:"maxDefense"`
	CurrentSpeed        int    `json:"currentSpeed"`
	MaxSpeed            int    `json:"maxSpeed"`
	CurrentIntelligence int    `json:"currentIntelligence"`
	MaxIntelligence     int    `json:"maxIntelligence"`
	CurrentMagic        int    `json:"currentMagic"`
	MaxMagic            int    `json:"maxMagic"`
}

type CollectResponse struct {
	Error string        `json:"error,omitempty"`
	Mnstr *models.Mnstr `json:"mnstr,omitempty"`
}

func HandleCollect(w http.ResponseWriter, r *http.Request) {
	var req CollectRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		sendCollectError(w, err, http.StatusBadRequest)
		return
	}

	session, err := models.GetSession(r.Context(), strings.Replace(r.Header.Get("Authorization"), "Bearer ", "", 1))
	if err != nil {
		sendCollectError(w, err, http.StatusUnauthorized)
		return
	}

	mnstr, _ := models.FindMnstrByQRCodeForUser(req.QRCode, session.UserID)

	if mnstr != nil {
		sendCollectSuccess(w, mnstr)
		return
	}

	mnstr = models.NewMnstr(req.QRCode, session.UserID)
	mnstr.Name = req.Name
	mnstr.CurrentHealth = req.CurrentHealth
	mnstr.MaxHealth = req.MaxHealth
	mnstr.CurrentAttack = req.CurrentAttack
	mnstr.MaxAttack = req.MaxAttack
	mnstr.CurrentDefense = req.CurrentDefense
	mnstr.MaxDefense = req.MaxDefense
	mnstr.CurrentSpeed = req.CurrentSpeed
	mnstr.MaxSpeed = req.MaxSpeed
	mnstr.CurrentIntelligence = req.CurrentIntelligence
	mnstr.MaxIntelligence = req.MaxIntelligence
	mnstr.CurrentMagic = req.CurrentMagic
	mnstr.MaxMagic = req.MaxMagic

	if err := mnstr.Create(); err != nil {
		sendCollectError(w, err, http.StatusInternalServerError)
		return
	}

	sendCollectSuccess(w, mnstr)
}

func sendCollectError(w http.ResponseWriter, err error, status int) {
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(CollectResponse{Error: err.Error()})
}

func sendCollectSuccess(w http.ResponseWriter, mnstr *models.Mnstr) {
	json.NewEncoder(w).Encode(CollectResponse{Mnstr: mnstr})
}
