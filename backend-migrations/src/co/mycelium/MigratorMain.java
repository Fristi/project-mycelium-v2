package co.mycelium;

import java.sql.DriverManager;
import javax.sql.DataSource;
import org.postgresql.ds.PGSimpleDataSource;
import org.flywaydb.core.Flyway;

class MigratorMain {

    public static PGSimpleDataSource createDataSource() {
        PGSimpleDataSource ds = new PGSimpleDataSource();

        // Retrieve environment variables with defaults
        String host = System.getenv().getOrDefault("PG_HOST", "localhost");
        String portStr = System.getenv().getOrDefault("PG_PORT", "5432");
        int port = Integer.parseInt(portStr);
        String user = System.getenv().getOrDefault("PG_USER", "postgres");
        String password = System.getenv().getOrDefault("PG_PASS", "postgres");
        String database = System.getenv().getOrDefault("PG_DB", "mycelium");

        // Configure the datasource
        ds.setServerNames(new String[]{host});
        ds.setPortNumbers(new int[]{port});
        ds.setUser(user);
        ds.setPassword(password);
        ds.setDatabaseName(database);

        return ds;
    }

    public static void main(String[] args)
    {
        Flyway
          .configure()
          .locations("migrations")
          .dataSource(createDataSource())
          .load()
          .migrate();
    }
}
