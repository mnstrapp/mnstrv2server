package manage

import (
	"log"
	"net/http"
	"strings"

	"github.com/mnstrapp/mnstrv2server/models"
)

type Handler struct{}

func NewHandler() Handler {
	return Handler{}
}

func (h Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	_, err := models.GetSession(r.Context(), strings.Replace(r.Header.Get("Authorization"), "Bearer ", "", 1))
	if err != nil {
		sendManageError(w, err, http.StatusUnauthorized)
		return
	}

	switch r.Method {
	case http.MethodPost, http.MethodPatch, http.MethodPut, http.MethodDelete:
		w.WriteHeader(404)
		log.Printf("Route not found: %s", r.URL.Path)
	case http.MethodGet:
		HandleManageList(w, r)
	}
}
