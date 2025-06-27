package auth

import (
	"encoding/json"
	"net/http"
	"strconv"

	"github.com/mnstrapp/mnstrv2server/models"
)

type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

type LoginResponse struct {
	Error   string         `json:"error"`
	Session models.Session `json:"session"`
	User    models.User    `json:"user"`
}

func HandleLogin(w http.ResponseWriter, r *http.Request) {
	var loginRequest LoginRequest
	err := json.NewDecoder(r.Body).Decode(&loginRequest)
	if err != nil {
		sendLoginError(w, err, http.StatusBadRequest)
		return
	}

	session, err := models.LogIn(loginRequest.Email, loginRequest.Password)
	if err != nil {
		sendLoginError(w, err, http.StatusInternalServerError)
		return
	}

	user, err := models.FindUserByID(session.UserID)
	if err != nil {
		sendLoginError(w, err, http.StatusInternalServerError)
		return
	}

	sendLoginSuccess(w, *session, *user)
}

func sendLoginError(w http.ResponseWriter, err error, status int) {
	loginResponse := LoginResponse{
		Error: err.Error(),
	}

	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Status", strconv.Itoa(status))
	json.NewEncoder(w).Encode(loginResponse)
}

func sendLoginSuccess(w http.ResponseWriter, session models.Session, user models.User) {
	loginResponse := LoginResponse{
		Error:   "",
		Session: session,
		User:    user,
	}

	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Status", strconv.Itoa(http.StatusOK))
	json.NewEncoder(w).Encode(loginResponse)
}
