local card={api_version=1,id="EX1_617",name="Deadly Shot",text="Destroy a random enemy minion.",set="EXPERT1",type="spell",class="hunter",rarity="common",cost=3,rules={can_play=function(ctx,self,current)return current and #ctx:enemy_minions(self)>0 end}}
function card.on_play(ctx,self)local m=ctx:enemy_minions(self);if #m>0 then ctx:random_entity(m,"destroy_selected")end end
function card.destroy_selected(ctx,self,target)cardlib.effects.destroy(ctx, target)end
return card
