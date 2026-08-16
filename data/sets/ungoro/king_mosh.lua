local card={api_version=1,id="UNG_933",name="King Mosh",text="<b>Battlecry:</b> Destroy all damaged minions.",set="UNGORO",type="minion",class="warrior",rarity="legendary",cost=7,attack=9,health=7,tags={"beast"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self) local targets={} for _,e in ipairs(ctx:minions()) do if ctx:entity(e).damage>0 then targets[#targets+1]=e end end cardlib.effects.destroy_all(ctx, targets) end
return card
