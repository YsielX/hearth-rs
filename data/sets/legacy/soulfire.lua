local card={api_version=1,id="EX1_308", rarity = "free",name="Soulfire",text="[x]Deal $4 damage.\nDiscard a random card.",set="LEGACY",type="spell",class="warlock",spell_school="fire",cost=1,target_mode="required",targets=function(ctx)return ctx:characters()end}
function card.on_play(ctx,self,target)cardlib.effects.damage(ctx, target,4);local h=ctx:hand(ctx:controller(self));if #h>0 then ctx:random_entity(h,"discard_chosen")end end
function card.discard_chosen(ctx,self,target)ctx:discard(ctx:controller(self),target)end
return card
