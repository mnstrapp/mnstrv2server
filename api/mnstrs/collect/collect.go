package collect

import (
	"encoding/json"
	"log"
	"net/http"
	"strings"

	"github.com/mnstrapp/mnstrv2server/models"
)

type CollectRequest struct {
	QRCode string `json:"qrCode"`
}

type CollectResponse struct {
	Error string        `json:"error"`
	Mnstr *models.Mnstr `json:"mnstr"`
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
	if err := mnstr.Create(); err != nil {
		sendCollectError(w, err, http.StatusInternalServerError)
		return
	}

	sendCollectSuccess(w, mnstr)
}

func sendCollectError(w http.ResponseWriter, err error, status int) {
	log.Printf("Error collecting mnstr: %v", err)
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(CollectResponse{Error: err.Error()})
}

func sendCollectSuccess(w http.ResponseWriter, mnstr *models.Mnstr) {
	json.NewEncoder(w).Encode(CollectResponse{Mnstr: mnstr})
}
