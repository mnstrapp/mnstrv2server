package users

import (
	"encoding/json"
	"net/http"

	"github.com/mnstrapp/mnstrv2server/models"
)

type ShowResponse struct {
	Error string       `json:"error"`
	User  *models.User `json:"user"`
}

func HandleShow(userId string, w http.ResponseWriter, r *http.Request) {
	user, err := models.FindUserByID(userId)
	if err != nil {
		sendShowError(w, err, http.StatusInternalServerError)
		return
	}

	sendShowSuccess(w, user)
}

func sendShowError(w http.ResponseWriter, err error, status int) {
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(ShowResponse{Error: err.Error()})
}

func sendShowSuccess(w http.ResponseWriter, user *models.User) {
	json.NewEncoder(w).Encode(ShowResponse{User: user})
}
