package pretty

import (
	"fmt"
	"time"
)

const (
	SHOW_SECONDS = time.Hour
)

func Date(t time.Time) string {
	d := time.Since(t)

	if d.Minutes() < 1. {
		return "Just now"
	}

	if d.Hours() < 1. {
		return fmt.Sprintf("%.0f minutes ago", d.Minutes())
	}

	if d.Hours() < 24. {
		return fmt.Sprintf("%.0f hours ago", d.Hours())
	}

	days := d.Hours() / 24
	if days < 31. {
		return fmt.Sprintf("%.0f days ago", days)
	}

	// todo: make month count more accurate
	years := days / 365
	if years < 1. {
		months := days / 30
		return fmt.Sprintf("%.0f months ago", months)
	}

	return fmt.Sprintf("%.0f years ago", years)
}
