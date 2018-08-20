import { Component, ViewChild, ElementRef, AfterViewInit } from '@angular/core';
import { Graph } from '../graph';
import { PruebaService } from '../../services/prueba.service';


@Component({
  selector: 'app-clients-graph',
  template: '<div #graph style="width: 100%; min-height: 320px"></div>',
})
export class SnrGraphComponent extends Graph implements AfterViewInit {
  @ViewChild('graph')
  protected _graphRef: ElementRef;

  public constructor(private _ps: PruebaService) {
    super();
    this._chartOptions = {
      title: {
        text: ''
      },
      tooltip: {
          trigger: 'axis'
      },
      xAxis: {
          data: 0
      },
      yAxis: {
          splitLine: {
              show: false
          }
      },
      toolbox: {
          left: 'center',
          feature: {
              dataZoom: {
                  yAxisIndex: 'none'
              },
              restore: {},
              saveAsImage: {}
          }
      },
      dataZoom: [{
          startValue: ''
      }, {
          type: 'inside'
      }],
      visualMap: {
          top: 10,
          right: 10,
          pieces: [{
              gt: 0,
              lte: 50,
              color: '#096'
          }, {
              gt: 50,
              lte: 100,
              color: '#ffde33'
          }, {
              gt: 100,
              lte: 150,
              color: '#ff9933'
          }, {
              gt: 150,
              lte: 200,
              color: '#cc0033'
          }, {
              gt: 200,
              lte: 300,
              color: '#660099'
          }, {
              gt: 300,
              color: '#7e0023'
          }],
          outOfRange: {
              color: '#999'
          }
      }
    };
  }

  ngAfterViewInit() {
    this._ps.getData().subscribe(data => {
      this.setSnrDataOnGraph(data);
      this.initGraph();
    });
  }

  public setSnrDataOnGraph(listSnrData: any[]) {
    const arrayData: any[] = [];
    const arrayTime: any[] = [];
    const node = listSnrData[0];
    const series = node.values;
    series.forEach(data => {
      arrayTime.push(data[0]);
      arrayData.push(data[4]);
    });
    this._chartOptions.title.text = node.name;
    this._chartOptions.xAxis.data = arrayTime;
    this._chartOptions.series = {
        name: node.columns[4],
        type: 'line',
        data: arrayData,
        markLine: {
            silent: true
        }
    };
  }
}
