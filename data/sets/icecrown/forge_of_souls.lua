local card={api_version=1,id="ICC_281",name="Forge of Souls",text="Draw 2 weapons from your deck.",set="ICECROWN",type="spell",class="warrior",rarity="common",cost=2}
function card.on_play(ctx,self)ctx:set_data(self,"draws",2);ctx:continue_with("draw_weapon")end
function card.draw_weapon(ctx,self)local pool={}for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="weapon"then pool[#pool+1]=e end end;if #pool>0 and ctx:get_data(self,"draws")>0 then ctx:random_entity(pool,"draw_selected_weapon")end end
function card.draw_selected_weapon(ctx,self,e)ctx:draw_entity(ctx:controller(self),e);local n=ctx:get_data(self,"draws")-1;ctx:set_data(self,"draws",n);if n>0 then ctx:continue_with("draw_weapon")end end
return card
