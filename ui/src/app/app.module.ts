import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';
import { HttpClientModule } from '@angular/common/http';

import { AppComponent } from './app.component';
import { GraphsModule } from './graphs/graphs.module';

import { PruebaService } from './services/prueba.service';


@NgModule({
  declarations: [
    AppComponent
  ],
  imports: [
    BrowserModule, HttpClientModule, GraphsModule
  ],
  providers: [PruebaService],
  bootstrap: [AppComponent]
})
export class AppModule { }
