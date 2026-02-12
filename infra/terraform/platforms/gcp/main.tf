terraform {
  required_version = ">= 1.6.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

locals {
  platform = "gcp"
}

#Node: module "network" {
#Node:   source      = "../../modules/gcp/network"
#Node:   project_id  = var.project_id
#Node:   environment = var.environment
#Node: }
#
#Node: module "run_services" {
#Node:   source       = "../../modules/gcp/cloudrun"
#Node:   service_name = var.service_name
#Node: }

output "skeleton_summary" {
  value = "terraform skeleton ready: platform=${local.platform}, env=${var.environment}, region=${var.region}, project=${var.project_id}"
}
