local card={api_version=1,id="LOOT_347",name="Kobold Apprentice",text="<b>Battlecry:</b> Deal 3 damage randomly split among all enemies.",set="LOOTAPALOOZA",type="minion",rarity="common",cost=3,attack=2,health=1,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)ctx:set_data(self,"kobold_missiles",3);ctx:continue_with("fire_kobold_missile")end
function card.fire_kobold_missile(ctx,self)if ctx:get_data(self,"kobold_missiles")>0 then local p=ctx:enemy_characters(self);if #p>0 then ctx:random_entity(p,"kobold_missile_hit")end end end
function card.kobold_missile_hit(ctx,self,target)ctx:damage(target,1);local n=ctx:get_data(self,"kobold_missiles")-1;ctx:set_data(self,"kobold_missiles",n);if n>0 then ctx:continue_with("fire_kobold_missile")end end
return card
