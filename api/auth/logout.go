package auth

import (
	"encoding/json"
	"net/http"

	"github.com/mnstrapp/mnstrv2server/models"
)

type LogoutResponse struct {
	Error string `json:"error"`
	Success bool `json:"success"`
}

func HandleLogout(w http.ResponseWriter, r *http.Request) {
	session, err := models.GetSession(r.Context(), r.Header.Get("Authorization"))
	if err != nil {
		sendLogoutError(w, err, http.StatusUnauthorized)
		return
	}

	models.Logout(session.Token)
	sendLogoutSuccess(w)
}

func sendLogoutError(w http.ResponseWriter, err error, status int) {
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(LogoutResponse{Error: err.Error(), Success: false})
}

func sendLogoutSuccess(w http.ResponseWriter) {
	json.NewEncoder(w).Encode(LogoutResponse{Success: true})
}