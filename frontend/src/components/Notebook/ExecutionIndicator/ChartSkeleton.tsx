import styles from "src/components/Notebook/ExecutionIndicator/ChartSkeleton.module.css";

const BAR_HEIGHTS = ["58%", "82%", "41%", "94%", "67%", "35%", "76%"];
const LEGEND_ITEMS = [0, 1, 2];

const BAR_STYLES = BAR_HEIGHTS.map((height, index) => ({
  height,
  animationDelay: `${index * 90}ms`,
}));

export function ChartSkeleton() {
  return (
    <div className={styles.skeleton} role="status" aria-label="Rendering chart">
      <div className={styles.plot}>
        {BAR_STYLES.map((style, index) => (
          <div key={index} className={styles.bar} style={style} />
        ))}
      </div>
      <div className={styles.legend}>
        {LEGEND_ITEMS.map((item) => (
          <div key={item} className={styles.legendItem} />
        ))}
      </div>
    </div>
  );
}
