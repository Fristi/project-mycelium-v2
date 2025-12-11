package co.mycelium.adapters.db

import co.mycelium.ports.{
  Repositories,
  StationLogRepository,
  StationMeasurementRepository,
  StationRepository
}
import doobie.ConnectionIO

object DoobieRepositories extends Repositories[ConnectionIO] {
  override def stationLog: StationLogRepository[ConnectionIO] =
    DoobieStationLogRepository
  override def stations: StationRepository[ConnectionIO] =
    DoobieStationRepository
  override def measurements: StationMeasurementRepository[ConnectionIO] =
    DoobieStationMeasurementRepository
}
