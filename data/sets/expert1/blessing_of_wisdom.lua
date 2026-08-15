local card={api_version=1,id="EX1_363",name="Blessing of Wisdom",text="Choose a minion. Whenever it attacks, draw a card.",set="EXPERT1",type="spell",class="paladin",rarity="common",spell_school="holy",cost=1,target_mode="required",targets=function(ctx)return ctx:minions()end}
function card.on_play(ctx,self,target)local p=ctx:controller(self);local key="wisdom_"..p;ctx:set_data(target,key,(ctx:get_data(target,key)or 0)+1);ctx:attach_script(target,"EX1_363")end
card.triggers={{event="attack",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.attacker==self end,effect=function(ctx,self)for p=0,1 do local n=ctx:get_data(self,"wisdom_"..p)or 0;if n>0 then ctx:draw(p,n)end end end}}
return card
