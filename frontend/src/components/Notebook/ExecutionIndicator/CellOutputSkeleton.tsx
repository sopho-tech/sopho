import styles from "src/components/Notebook/ExecutionIndicator/CellOutputSkeleton.module.css";

const COLUMN_COUNT = 4;
const ROW_COUNT = 5;

const WIDTH_CLASSES = [
  styles.widthWide,
  styles.widthNarrow,
  styles.widthMedium,
];

const COLUMNS = Array.from({ length: COLUMN_COUNT }, (_, index) => index);
const ROWS = Array.from({ length: ROW_COUNT }, (_, index) => index);

export function CellOutputSkeleton() {
  return (
    <div
      className={styles.skeleton}
      role="status"
      aria-label="Loading query results"
    >
      <div className={styles.headerRow}>
        {COLUMNS.map((column) => (
          <div key={column} className={styles.cell}>
            <div className={`${styles.bar} ${styles.widthMedium}`} />
          </div>
        ))}
      </div>
      {ROWS.map((row) => (
        <div key={row} className={styles.row}>
          {COLUMNS.map((column) => (
            <div key={column} className={styles.cell}>
              <div
                className={`${styles.bar} ${WIDTH_CLASSES[(row + column) % WIDTH_CLASSES.length]}`}
              />
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
