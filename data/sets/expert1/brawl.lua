local card={api_version=1,id="EX1_407",name="Brawl",text="Destroy all minions except one. <i>(chosen randomly)</i>",set="EXPERT1",type="spell",class="warrior",rarity="epic",cost=5,rules={can_play=function(ctx,self,current)return current and #ctx:minions()>1 end}}
function card.on_play(ctx)local m=ctx:minions();if #m>0 then ctx:random_entity(m,"save_selected")end end
function card.save_selected(ctx,self,s)local r={};for _,e in ipairs(ctx:minions())do if e~=s then r[#r+1]=e end end;cardlib.effects.destroy_all(ctx, r)end
return card
