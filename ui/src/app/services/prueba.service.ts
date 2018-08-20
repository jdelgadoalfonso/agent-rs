import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';


@Injectable()
export class PruebaService {
  constructor(private http: HttpClient) { }

  private readonly _configUrl = 'https://localhost:8443/';

  public getData() {
    return this.http.get<any[]>(this._configUrl);
  }
}
