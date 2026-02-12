terraform {
  required_version = ">= 1.6.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.region

  default_tags {
    tags = {
      Service     = var.service_name
      Environment = var.environment
      ManagedBy   = "terraform"
      Platform    = "aws"
    }
  }
}

locals {
  platform = "aws"
}

#Node: module "network" {
#Node:   source      = "../../modules/aws/network"
#Node:   environment = var.environment
#Node: }
#
#Node: module "api_runtime" {
#Node:   source       = "../../modules/aws/runtime"
#Node:   service_name = var.service_name
#Node: }

output "skeleton_summary" {
  value = "terraform skeleton ready: platform=${local.platform}, env=${var.environment}, region=${var.region}"
}
