{{- define "postal-converter-ja.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "postal-converter-ja.fullname" -}}
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

{{- define "postal-converter-ja.labels" -}}
app.kubernetes.io/name: {{ include "postal-converter-ja.name" . }}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "postal-converter-ja.selectorLabels" -}}
app.kubernetes.io/name: {{ include "postal-converter-ja.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "postal-converter-ja.secretName" -}}
{{- if .Values.secret.name -}}
{{- .Values.secret.name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-secret" (include "postal-converter-ja.fullname" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
