package handlers

import (
	"html/template"
	"net/http"
)

type Handler struct {
	Tmpl *template.Template
}

func (h *Handler) Index(w http.ResponseWriter, r *http.Request) {

	data := struct {
		Greeting string
	}{
		Greeting: "hello and welcome",
	}

	err := h.Tmpl.ExecuteTemplate(w, "index.gohtml", data)
	if err != nil {
		http.Error(w, "Error executing template", http.StatusInternalServerError)
		return
	}

}
