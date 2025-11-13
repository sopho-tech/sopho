import { useEffect, useRef, useMemo } from "react";
import {
  init,
  type EChartsInitOpts,
  getInstanceByDom,
  EChartsType,
  EChartsOption,
  SetOptionOpts,
} from "echarts";
import debounce from "lodash.debounce";
import ChartStyles from "src/components/Chart/Chart.module.css";

export enum ChartType {
  BAR = "BAR",
  LINE = "LINE",
  PIE = "PIE",
  SCATTER = "SCATTER",
}

export type ChartProps = {
  option: EChartsOption;
};

export function Chart({ option }: ChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const eChartsInitOpts: EChartsInitOpts = {
    locale: "EN",
    renderer: "canvas",
    useDirtyRect: false,
    useCoarsePointer: false,
    ssr: false,
    width: "auto",
    height: "auto",
  };
  const setOptionOpts: SetOptionOpts = {
    notMerge: false,
    lazyUpdate: false,
    silent: false,
    replaceMerge: undefined,
    transition: undefined,
  };
  const resizeChart = useMemo(
    () =>
      debounce(() => {
        if (chartContainerRef.current) {
          const chart = getInstanceByDom(chartContainerRef.current);
          if (!chart) throw Error("chart is undefined");
          chart.resize();
        }
      }, 50),
    []
  );

  useEffect(() => {
    const chart = init(chartContainerRef.current, "theme", eChartsInitOpts);

    const resizeObserver = new ResizeObserver(() => {
      resizeChart();
    });
    resizeObserver.observe(chartContainerRef.current as Element);

    return () => {
      if (chartContainerRef.current) {
        resizeObserver.unobserve(chartContainerRef.current);
      }
      resizeObserver.disconnect();
      chart.dispose();
    };
  }, [chartContainerRef, eChartsInitOpts]);

  useEffect(() => {
    const chart: EChartsType | undefined = getInstanceByDom(
      chartContainerRef.current as HTMLElement
    );
    if (!chart) throw Error("chart is undefined");
    chart.setOption(option, setOptionOpts);
  }, [option, setOptionOpts]);

  return <div className={ChartStyles.container} ref={chartContainerRef}></div>;
}
