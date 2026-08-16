local card={api_version=1,id="EX1_391",name="Slam",text="Deal $2 damage to a minion. If it survives, draw a card.",set="EXPERT1",type="spell",class="warrior",rarity="common",cost=1,target_mode="required",targets=function(ctx)return ctx:minions()end}
function card.on_play(ctx,self,target)cardlib.effects.damage(ctx, target,2);ctx:continue_with_entity("check_survivor",target)end
function card.check_survivor(ctx,self,target)if ctx:entity(target).zone=="board"then ctx:draw(ctx:controller(self),1)end end
return card
