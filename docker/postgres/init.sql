-- Meridian GIS Platform - PostgreSQL Initialization Script
-- Creates necessary extensions and initial database schema

-- Enable PostGIS extension for GIS functionality
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

-- Enable other useful extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gist";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS meridian;
CREATE SCHEMA IF NOT EXISTS accessibility;

-- Set default search path
ALTER DATABASE meridian_gis SET search_path TO meridian, accessibility, public;

-- Grant permissions
GRANT ALL PRIVILEGES ON SCHEMA meridian TO meridian;
GRANT ALL PRIVILEGES ON SCHEMA accessibility TO meridian;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA meridian TO meridian;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA accessibility TO meridian;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA meridian TO meridian;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA accessibility TO meridian;

-- Set default privileges for future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA meridian GRANT ALL ON TABLES TO meridian;
ALTER DEFAULT PRIVILEGES IN SCHEMA meridian GRANT ALL ON SEQUENCES TO meridian;
ALTER DEFAULT PRIVILEGES IN SCHEMA accessibility GRANT ALL ON TABLES TO meridian;
ALTER DEFAULT PRIVILEGES IN SCHEMA accessibility GRANT ALL ON SEQUENCES TO meridian;

-- Log initialization
DO $$
BEGIN
    RAISE NOTICE 'Meridian GIS Platform database initialized successfully';
END $$;
