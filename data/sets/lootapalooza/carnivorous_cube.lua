local card={api_version=1,id="LOOT_161",name="Carnivorous Cube",text="<b>Battlecry:</b> Destroy\na friendly minion.\n<b>Deathrattle:</b> Summon 2 copies of it.",set="LOOTAPALOOZA",type="minion",rarity="epic",cost=5,attack=4,health=6,keywords={"battlecry","deathrattle"},target_mode="required_if_available",targets=function(ctx,self)local r={};for _,e in ipairs(ctx:friendly_minions(self))do if e~=self then r[#r+1]=e end end;return r end}
function card.on_battlecry(ctx,self,target)if target then ctx:set_data(self,"cube_victim",target);cardlib.effects.destroy(ctx, target)end end
function card.on_deathrattle(ctx,self)local e=ctx:get_data(self,"cube_victim");if e>0 then ctx:summon_fresh_copy(e);ctx:summon_fresh_copy(e)end end
return card
