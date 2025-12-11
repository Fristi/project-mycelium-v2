CREATE TABLE station_profile (
    station_id UUID PRIMARY KEY REFERENCES stations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    light_mmol_start INT NOT NULL,
    light_mmol_end INT NOT NULL,
    light_lux_start INT NOT NULL,
    light_lux_end INT NOT NULL,
    temperature_start INT NOT NULL,
    temperature_end INT NOT NULL,
    humidity_start INT NOT NULL,
    humidity_end INT NOT NULL,
    soil_moisture_start INT NOT NULL,
    soil_moisture_end INT NOT NULL,
    soil_ec_start INT NOT NULL,
    soil_ec_end INT NOT NULL
);