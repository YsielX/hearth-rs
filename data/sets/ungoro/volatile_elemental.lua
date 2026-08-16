local card={api_version=1,id="UNG_818",name="Volatile Elemental",text="<b>Deathrattle:</b> Deal 3 damage to a random enemy minion.",set="UNGORO",type="minion",class="neutral",rarity="common",cost=2,attack=3,health=1,tags={"elemental"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self) local t={} for _,e in ipairs(ctx:enemy_characters(self)) do if ctx:entity(e).type=="minion" then t[#t+1]=e end end if #t>0 then ctx:random_entity(t,"volatile_hit") end end
function card.volatile_hit(ctx,self,target) cardlib.effects.damage(ctx, target,3) end
return card
