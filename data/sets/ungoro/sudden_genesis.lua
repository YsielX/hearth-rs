local card={api_version=1,id="UNG_927",name="Sudden Genesis",text="Summon copies of your damaged minions.",set="UNGORO",type="spell",class="warrior",rarity="epic",spell_school="nature",cost=4}
card.rules={can_play=function(ctx,self) return #ctx:board(ctx:controller(self))<7 end}
function card.on_play(ctx,self) local targets={} for _,e in ipairs(ctx:friendly_minions(self)) do if ctx:entity(e).damage>0 then targets[#targets+1]=e end end for _,e in ipairs(targets) do ctx:summon_copy(ctx:controller(self),e) end end
return card
