import { ElementRef } from '@angular/core';
import * as echarts from 'echarts';


export abstract class Graph {
  protected _graphRef: ElementRef;
  protected _chart: echarts.ECharts;
  protected _chartOptions;
  protected _keepZoom;
  protected _optimizeZoom;
  protected _exportPDFoptions;

  get chartOptions() {
    return this._chartOptions;
  }

  protected initGraph(): void {
    if (this._graphRef) {
      this.disposeDivGraphElement();
      this._chart = echarts.init(<HTMLDivElement> this._graphRef.nativeElement);
      // this.setImgDate();
      this.setGraphOptions();
    }
  }

  public keepZoom(data: Object, options: any) {
    const _chart = Object.assign(Object.create(data.constructor.prototype), data);
    const option = _chart.getOption();
    const startZoom = option.dataZoom[0].start;
    const endZoom = option.dataZoom[0].end;
    options.dataZoom.end = endZoom;
    options.dataZoom.start = startZoom;
  }

  public optimizeZoom(data: Object, options: any, lengthList: number) {
    const chart = Object.assign(Object.create(data.constructor.prototype), data);
    const option = chart.getOption();
    const endZoom = option.dataZoom[0].end;
    if (lengthList >= 10) {
      options.dataZoom.show = true;
      options.dataZoom.end = 35;
    } else {
      options.dataZoom.show = false;
      options.dataZoom.start = 0;
      options.dataZoom.end = 100;
    }
  }

  public reloadGraph(): void {
    setTimeout(() => this.initGraph());
  }

  public setGraphOptions(): void {
    if (!this.isGraphDispose()) {
      this._chart.setOption(this._chartOptions, false, true);
    }
  }

  public resizeGraph(): void {
    this._chart.resize();
  }

  public clearGraph(): void {
    this._chart.dispose();
  }

  public disposeDivGraphElement(): void {
    echarts.dispose(this._graphRef.nativeElement);
  }

  public isGraphDispose(): boolean {
    return this._chart.isDisposed();
  }

  public getCanvasChartElement(): HTMLCanvasElement {
    const element: HTMLElement = this._graphRef.nativeElement;
    const canvas: HTMLCanvasElement = element.querySelector('canvas');
    return canvas;
  }

  public toogleOptionsToExport(active: boolean) {
    this._chartOptions.toolbox.show = active;
    if (this._chartOptions.dataZoom != null) {
      this._chartOptions.dataZoom.show = active;
      this._chartOptions.dataZoom.start = this._chartOptions.dataZoom.start;
      this._chartOptions.dataZoom.end = this._chartOptions.dataZoom.end;
    }
    this.setGraphOptions();
  }

  public getExportPDFoptions(): any {
    return this._exportPDFoptions;
  }

  public setExportPDFoptions(exportPDFoptions: any) {
    this._exportPDFoptions = exportPDFoptions;
  }
  /*
  public setImgDate() {
    let imgName = this._chartOptions.toolbox.feature['saveAsImage'].name;
    imgName = `${imgName}_${DateUtils.exportFileDate()}`;
    this._chartOptions.toolbox.feature['saveAsImage'].name = imgName;
  }
  */
}
