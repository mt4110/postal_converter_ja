variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "dev"

  validation {
    condition     = contains(["dev", "stg", "prod"], var.environment)
    error_message = "environment must be one of: dev, stg, prod."
  }
}

variable "service_name" {
  description = "Service name used for resource naming/tags"
  type        = string
  default     = "postal-converter-ja"
}

variable "project_id" {
  description = "GCP project ID"
  type        = string
  default     = "replace-me"
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "asia-northeast1"
}
