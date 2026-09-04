{{/* Chart name, overridable via nameOverride. */}}
{{- define "caby.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/* Fully qualified app name. */}}
{{- define "caby.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{- define "caby.backend.fullname" -}}
{{- printf "%s-service" (include "caby.fullname" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "caby.web.fullname" -}}
{{- printf "%s-web" (include "caby.fullname" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "caby.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "caby.labels" -}}
helm.sh/chart: {{ include "caby.chart" . }}
{{ include "caby.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: caby
{{- end -}}

{{- define "caby.selectorLabels" -}}
app.kubernetes.io/name: {{ include "caby.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "caby.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "caby.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "caby.backend.image" -}}
{{- $tag := default .Chart.AppVersion .Values.backend.image.tag -}}
{{- printf "%s:%s" .Values.backend.image.repository $tag -}}
{{- end -}}

{{- define "caby.web.image" -}}
{{- $tag := default .Chart.AppVersion .Values.web.image.tag -}}
{{- printf "%s:%s" .Values.web.image.repository $tag -}}
{{- end -}}

{{- define "caby.configSecretName" -}}
{{- if .Values.config.existingSecret -}}
{{- .Values.config.existingSecret -}}
{{- else -}}
{{- printf "%s-config" (include "caby.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "caby.configSecretKey" -}}
{{- default "config.yaml" .Values.config.key -}}
{{- end -}}

{{- define "caby.pvcName" -}}
{{- if .Values.persistence.existingClaim -}}
{{- .Values.persistence.existingClaim -}}
{{- else -}}
{{- printf "%s-data" (include "caby.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/* Public URL scheme, derived from whether TLS is enabled on the ingress. */}}
{{- define "caby.scheme" -}}
{{- if .Values.ingress.tls -}}https{{- else -}}http{{- end -}}
{{- end -}}

{{/* Public origin of the web frontend (no trailing path). */}}
{{- define "caby.webUrl" -}}
{{- if .Values.backend.frontendUrl -}}
{{- .Values.backend.frontendUrl -}}
{{- else -}}
{{- printf "%s://%s" (include "caby.scheme" .) (required "Set ingress.web.host (public hostname) or backend.frontendUrl" .Values.ingress.web.host) -}}
{{- end -}}
{{- end -}}

{{/* Optional base path the API is mounted under: /api on a shared host (its own path space), empty on a dedicated API host. Override with ingress.api.basePath (empty string is a valid override). */}}
{{- define "caby.apiBasePath" -}}
{{- if not (kindIs "invalid" .Values.ingress.api.basePath) -}}
{{- .Values.ingress.api.basePath -}}
{{- else if eq (include "caby.singleHost" .) "true" -}}
/api
{{- end -}}
{{- end -}}

{{/* Public base URL of the backend API (origin + base path, no /v0). CABY_BACKEND_URL is set to this; the backend appends /v0. */}}
{{- define "caby.apiUrl" -}}
{{- if .Values.backend.backendUrl -}}
{{- .Values.backend.backendUrl -}}
{{- else -}}
{{- $host := coalesce .Values.ingress.api.host .Values.ingress.web.host -}}
{{- printf "%s://%s%s" (include "caby.scheme" .) (required "Set ingress.web.host (public hostname) or backend.backendUrl" $host) (include "caby.apiBasePath" .) -}}
{{- end -}}
{{- end -}}

{{/* Value of PUBLIC_API_BASE the browser uses (backend base URL + /v0). */}}
{{- define "caby.publicApiBase" -}}
{{- if .Values.web.publicApiBase -}}
{{- .Values.web.publicApiBase -}}
{{- else -}}
{{- printf "%s/v0" (include "caby.apiUrl" .) -}}
{{- end -}}
{{- end -}}

{{/* "true" when the API shares the web host (single-host, path-based, same-origin). */}}
{{- define "caby.singleHost" -}}
{{- if or (not .Values.ingress.api.host) (eq (.Values.ingress.api.host | toString) (.Values.ingress.web.host | toString)) -}}true{{- end -}}
{{- end -}}
