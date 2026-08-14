local card={api_version=1,id="LOOT_211",name="Elven Minstrel",text="<b>Combo:</b> Draw 2 minions from your deck.",set="LOOTAPALOOZA",type="minion",class="rogue",rarity="rare",cost=4,attack=3,health=3,keywords={"combo"}}
function card.on_combo(ctx,self)ctx:set_data(self,"minstrel_left",2);ctx:continue_with("minstrel_choose")end
function card.minstrel_choose(ctx,self)if ctx:get_data(self,"minstrel_left")<=0 then return end;local p={};for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="minion"then p[#p+1]=e end end;if #p>0 then ctx:random_entity(p,"minstrel_draw")end end
function card.minstrel_draw(ctx,self,e)ctx:draw_entity(ctx:controller(self),e);local n=ctx:get_data(self,"minstrel_left")-1;ctx:set_data(self,"minstrel_left",n);if n>0 then ctx:continue_with("minstrel_choose")end end
return card
