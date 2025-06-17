#!/bin/bash
# Production Deployment Script for WhatsApp Engine
#
# This script handles the complete production deployment process

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="whatsapp-engine"
DOCKER_COMPOSE_FILE="docker/docker-compose.production.yml"
ENV_FILE="docker/.env.production"
BACKUP_DIR="backups/$(date +%Y%m%d_%H%M%S)"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if Docker is installed and running
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        log_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    # Check if Docker Compose is available
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if environment file exists
    if [ ! -f "$ENV_FILE" ]; then
        log_warning "Environment file $ENV_FILE not found. Creating from template..."
        cp "${ENV_FILE}.example" "$ENV_FILE" 2>/dev/null || true
        log_warning "Please edit $ENV_FILE with your production settings before continuing."
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

validate_environment() {
    log_info "Validating environment configuration..."
    
    source "$ENV_FILE"
    
    # Check critical environment variables
    if [ "$WHATSAPP_API_TOKEN" = "your-secure-production-token-change-this" ]; then
        log_error "Please set a secure API token in $ENV_FILE"
        exit 1
    fi
    
    if [ "$GRAFANA_PASSWORD" = "secure-grafana-password-change-this" ]; then
        log_warning "Please set a secure Grafana password in $ENV_FILE"
    fi
    
    log_success "Environment validation passed"
}

create_backup() {
    log_info "Creating backup of current deployment..."
    
    mkdir -p "$BACKUP_DIR"
    
    # Backup current containers if they exist
    if docker-compose -f "$DOCKER_COMPOSE_FILE" ps -q whatsapp-engine &> /dev/null; then
        docker-compose -f "$DOCKER_COMPOSE_FILE" logs whatsapp-engine > "$BACKUP_DIR/whatsapp-engine.log" 2>&1 || true
    fi
    
    # Backup volumes
    docker run --rm -v whatsapp-engine_whatsapp_data:/data -v "$(pwd)/$BACKUP_DIR":/backup alpine tar czf /backup/whatsapp_data.tar.gz -C /data . 2>/dev/null || true
    
    log_success "Backup created in $BACKUP_DIR"
}

build_and_deploy() {
    log_info "Building and deploying WhatsApp Engine..."
    
    # Pull latest changes (if in git repository)
    if [ -d ".git" ]; then
        log_info "Pulling latest changes..."
        git pull origin main || true
    fi
    
    # Build and start services
    log_info "Building Docker images..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" --env-file "$ENV_FILE" build --no-cache
    
    log_info "Starting services..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" --env-file "$ENV_FILE" up -d
    
    log_success "Deployment completed"
}

wait_for_health() {
    log_info "Waiting for services to become healthy..."
    
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f http://localhost:3000/health &> /dev/null; then
            log_success "WhatsApp Engine is healthy"
            return 0
        fi
        
        log_info "Attempt $attempt/$max_attempts - waiting for health check..."
        sleep 10
        ((attempt++))
    done
    
    log_error "Health check failed after $max_attempts attempts"
    return 1
}

run_smoke_tests() {
    log_info "Running smoke tests..."
    
    # Test health endpoint
    if curl -f http://localhost:3000/health &> /dev/null; then
        log_success "Health endpoint: OK"
    else
        log_error "Health endpoint: FAILED"
        return 1
    fi
    
    # Test metrics endpoint
    if curl -f http://localhost:3000/metrics &> /dev/null; then
        log_success "Metrics endpoint: OK"
    else
        log_error "Metrics endpoint: FAILED"
        return 1
    fi
    
    # Test readiness endpoint
    if curl -f http://localhost:3000/ready &> /dev/null; then
        log_success "Readiness endpoint: OK"
    else
        log_error "Readiness endpoint: FAILED"
        return 1
    fi
    
    log_success "All smoke tests passed"
}

show_status() {
    log_info "Deployment status:"
    echo
    docker-compose -f "$DOCKER_COMPOSE_FILE" ps
    echo
    log_info "Service URLs:"
    echo "  - WhatsApp Engine API: http://localhost:3000"
    echo "  - API Documentation: http://localhost:3000/swagger-ui/"
    echo "  - Health Check: http://localhost:3000/health"
    echo "  - Metrics: http://localhost:3000/metrics"
    echo "  - Prometheus: http://localhost:9090"
    echo "  - Grafana: http://localhost:3001 (admin/$(source $ENV_FILE && echo $GRAFANA_PASSWORD))"
    echo
}

cleanup_old_images() {
    log_info "Cleaning up old Docker images..."
    docker image prune -f || true
    log_success "Cleanup completed"
}

# Main deployment process
main() {
    log_info "Starting WhatsApp Engine production deployment..."
    echo
    
    check_prerequisites
    validate_environment
    create_backup
    build_and_deploy
    
    if wait_for_health; then
        run_smoke_tests
        show_status
        cleanup_old_images
        
        log_success "🚀 WhatsApp Engine deployed successfully!"
        log_info "Check the service logs with: docker-compose -f $DOCKER_COMPOSE_FILE logs -f"
    else
        log_error "Deployment failed - services are not healthy"
        log_info "Check logs with: docker-compose -f $DOCKER_COMPOSE_FILE logs"
        exit 1
    fi
}

# Handle script arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "stop")
        log_info "Stopping WhatsApp Engine..."
        docker-compose -f "$DOCKER_COMPOSE_FILE" down
        log_success "WhatsApp Engine stopped"
        ;;
    "restart")
        log_info "Restarting WhatsApp Engine..."
        docker-compose -f "$DOCKER_COMPOSE_FILE" restart
        wait_for_health && log_success "WhatsApp Engine restarted successfully"
        ;;
    "logs")
        docker-compose -f "$DOCKER_COMPOSE_FILE" logs -f
        ;;
    "status")
        show_status
        ;;
    "update")
        log_info "Updating WhatsApp Engine..."
        docker-compose -f "$DOCKER_COMPOSE_FILE" pull
        docker-compose -f "$DOCKER_COMPOSE_FILE" up -d
        wait_for_health && log_success "WhatsApp Engine updated successfully"
        ;;
    "backup")
        create_backup
        ;;
    *)
        echo "Usage: $0 {deploy|stop|restart|logs|status|update|backup}"
        echo
        echo "Commands:"
        echo "  deploy  - Full deployment (default)"
        echo "  stop    - Stop all services"
        echo "  restart - Restart all services"
        echo "  logs    - Show service logs"
        echo "  status  - Show deployment status"
        echo "  update  - Update to latest images"
        echo "  backup  - Create backup only"
        exit 1
        ;;
esac
