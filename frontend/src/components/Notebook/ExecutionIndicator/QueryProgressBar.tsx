import styles from "src/components/Notebook/ExecutionIndicator/QueryProgressBar.module.css";

export function QueryProgressBar() {
  return (
    <div
      className={styles.track}
      role="progressbar"
      aria-label="Running query"
      aria-busy="true"
    >
      <div className={styles.indicator} />
    </div>
  );
}
