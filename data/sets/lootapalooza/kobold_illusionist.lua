local card={api_version=1,id="LOOT_412",name="Kobold Illusionist",text="<b>Deathrattle:</b> Summon a 1/1 copy of a minion from your hand.",set="LOOTAPALOOZA",type="minion",class="rogue",rarity="rare",cost=5,attack=3,health=3,keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)local p={};for _,e in ipairs(ctx:hand(ctx:controller(self)))do if ctx:entity(e).type=="minion"then p[#p+1]=e end end;if #p>0 then ctx:random_entity(p,"summon_illusion")end end
function card.summon_illusion(ctx,self,e)ctx:summon_copy_with_stats(ctx:controller(self),e,1,1)end
return card
