package collect

import (
	"encoding/json"
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
	User  *models.User  `json:"user"`
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
		user, err := models.FindUserByID(session.UserID)
		if err != nil {
			sendCollectError(w, err, http.StatusInternalServerError)
			return
		}

		sendCollectSuccess(w, mnstr, user)
		return
	}

	mnstr = models.NewMnstr(req.QRCode, session.UserID)
	if err := mnstr.Create(); err != nil {
		sendCollectError(w, err, http.StatusInternalServerError)
		return
	}

	user, err := models.FindUserByID(session.UserID)
	if err != nil {
		sendCollectError(w, err, http.StatusInternalServerError)
		return
	}

	sendCollectSuccess(w, mnstr, user)
}

func sendCollectError(w http.ResponseWriter, err error, status int) {
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(CollectResponse{Error: err.Error()})
}

func sendCollectSuccess(w http.ResponseWriter, mnstr *models.Mnstr, user *models.User) {
	json.NewEncoder(w).Encode(CollectResponse{Mnstr: mnstr, User: user})
}
