ALTER TABLE stations DROP COLUMN watering_schedule;


ALTER TABLE stations 
ALTER COLUMN description DROP NOT NULL,
ALTER COLUMN location DROP NOT NULL;

ALTER TABLE station_measurements
ALTER COLUMN battery_voltage TYPE decimal(8,3),
ALTER COLUMN temperature TYPE decimal(8,3),
ALTER COLUMN humidity TYPE decimal(8,3),
ALTER COLUMN lux TYPE decimal(8,3),
ALTER COLUMN soil_pf TYPE decimal(8,3),
ALTER COLUMN tank_pf TYPE decimal(8,3);
