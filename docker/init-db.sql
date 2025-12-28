-- Meridian GIS Database Initialization Script
-- This script is automatically run when the PostgreSQL container starts for the first time

-- Enable PostGIS extension
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;
CREATE EXTENSION IF NOT EXISTS postgis_raster;

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable full-text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Enable hstore for key-value pairs
CREATE EXTENSION IF NOT EXISTS hstore;

-- Create schemas
CREATE SCHEMA IF NOT EXISTS meridian;
CREATE SCHEMA IF NOT EXISTS audit;

-- Set search path
SET search_path TO meridian, public, postgis;

-- Grant permissions to meridian user
GRANT ALL ON SCHEMA meridian TO meridian;
GRANT ALL ON SCHEMA audit TO meridian;
GRANT ALL ON ALL TABLES IN SCHEMA meridian TO meridian;
GRANT ALL ON ALL SEQUENCES IN SCHEMA meridian TO meridian;
GRANT ALL ON ALL FUNCTIONS IN SCHEMA meridian TO meridian;

-- Create spatial reference system table if needed
-- PostGIS typically includes this, but we ensure it exists
CREATE TABLE IF NOT EXISTS meridian.spatial_ref_sys (
    srid INTEGER NOT NULL PRIMARY KEY,
    auth_name VARCHAR(256),
    auth_srid INTEGER,
    srtext VARCHAR(2048),
    proj4text VARCHAR(2048)
);

-- Add commonly used spatial reference systems
-- WGS 84 (EPSG:4326) - Geographic coordinate system
-- Web Mercator (EPSG:3857) - Popular web mapping projection
-- These are typically already in PostGIS, but we ensure they exist

-- Create audit log function
CREATE OR REPLACE FUNCTION audit.log_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit.log (table_name, operation, row_id, new_data, changed_at)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, row_to_json(NEW), CURRENT_TIMESTAMP);
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit.log (table_name, operation, row_id, old_data, new_data, changed_at)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, row_to_json(OLD), row_to_json(NEW), CURRENT_TIMESTAMP);
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO audit.log (table_name, operation, row_id, old_data, changed_at)
        VALUES (TG_TABLE_NAME, TG_OP, OLD.id, row_to_json(OLD), CURRENT_TIMESTAMP);
        RETURN OLD;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Create audit log table
CREATE TABLE IF NOT EXISTS audit.log (
    id BIGSERIAL PRIMARY KEY,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL,
    row_id UUID,
    old_data JSONB,
    new_data JSONB,
    changed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    changed_by TEXT DEFAULT CURRENT_USER
);

-- Create index on audit log
CREATE INDEX IF NOT EXISTS idx_audit_log_table_name ON audit.log(table_name);
CREATE INDEX IF NOT EXISTS idx_audit_log_changed_at ON audit.log(changed_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_row_id ON audit.log(row_id);

-- Log initialization
DO $$
BEGIN
    RAISE NOTICE 'Meridian GIS database initialized successfully';
    RAISE NOTICE 'PostGIS version: %', PostGIS_Full_Version();
END $$;
