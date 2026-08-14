local card={api_version=1,id="LOOT_033",name="Cavern Shinyfinder",text="<b>Battlecry:</b> Draw a weapon from your deck.",set="LOOTAPALOOZA",type="minion",class="rogue",rarity="common",cost=2,attack=3,health=1,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local p={};for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="weapon"then p[#p+1]=e end end;if #p>0 then ctx:random_entity(p,"draw_shiny_weapon")end end
function card.draw_shiny_weapon(ctx,self,e)ctx:draw_entity(ctx:controller(self),e)end
return card
