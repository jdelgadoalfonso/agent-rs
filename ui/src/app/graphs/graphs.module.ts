import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';

import { SnrGraphComponent } from './clients/clients.component';

@NgModule({
  declarations: [
    SnrGraphComponent
  ],
  imports: [
    BrowserModule
  ],
  exports: [SnrGraphComponent],
  providers: []
})
export class GraphsModule { }
