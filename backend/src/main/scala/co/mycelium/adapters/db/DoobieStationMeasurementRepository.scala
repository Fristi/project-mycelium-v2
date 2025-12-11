package co.mycelium.adapters.db

import cats.tagless.{Derive, FunctorK}
import co.mycelium.domain.*
import co.mycelium.ports.StationMeasurementRepository
import doobie.*
import doobie.implicits.*
import doobie.postgres.implicits.*

import java.util.UUID
import scala.annotation.experimental

object DoobieStationMeasurementRepository extends StationMeasurementRepository[ConnectionIO] {
  override def insertMany(
      stationId: UUID,
      measurements: List[StationMeasurement]
  ): ConnectionIO[Int] =
    Update[(UUID, StationMeasurement)](
      "insert into station_measurements (station_id, occurred_on, battery_voltage, temperature, humidity, lux, soil_pf, tank_pf) values (?, ?, ?, ?, ?, ?, ?, ?)"
    )
      .updateMany(measurements.map(x => (stationId, x)))

  override def avg(
      stationId: UUID,
      period: MeasurementPeriod
  ): ConnectionIO[List[StationMeasurement]] = {

    val timeBucket = period match {
      case MeasurementPeriod.LastTwentyFourHours => fr"time_bucket('15 minutes', occurred_on)"
      case MeasurementPeriod.LastSevenDays       => fr"time_bucket('1 day', occurred_on)"
      case MeasurementPeriod.LastTwoWeeks        => fr"time_bucket('1 day', occurred_on)"
      case MeasurementPeriod.LastMonth           => fr"time_bucket('1 day', occurred_on)"
    }

    val limit = period match {
      case MeasurementPeriod.LastTwentyFourHours => 24
      case MeasurementPeriod.LastSevenDays       => 7
      case MeasurementPeriod.LastTwoWeeks        => 14
      case MeasurementPeriod.LastMonth           => 31
    }

    fr"SELECT $timeBucket AS bucket, avg(battery_voltage) as battery_voltage, avg(temperature) as temperature, avg(humidity) as humidity, avg(lux) as lux, avg(soil_pf) as soil_pf, avg(tank_pf) as tank_pf FROM station_measurements GROUP BY bucket ORDER BY bucket ASC LIMIT $limit"
      .query[StationMeasurement]
      .to[List]
  }
}
