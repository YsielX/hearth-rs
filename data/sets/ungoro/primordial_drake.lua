local card={api_version=1,id="UNG_848",name="Primordial Drake",text="[x]<b>Taunt</b>\n<b>Battlecry:</b> Deal 2 damage\nto all other minions.",set="UNGORO",type="minion",class="neutral",rarity="epic",cost=8,attack=4,health=8,tags={"dragon"},keywords={"taunt","battlecry"}}
function card.on_battlecry(ctx,self) local targets={} for _,e in ipairs(ctx:minions()) do if e~=self then targets[#targets+1]=e end end ctx:damage_all(targets,2) end
return card
