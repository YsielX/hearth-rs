local card={api_version=1,id="ICC_469",name="Unwilling Sacrifice",text="Choose a friendly minion. Destroy it and a random enemy minion.",set="ICECROWN",type="spell",class="warlock",rarity="rare",spell_school="shadow",cost=3,target_mode="required",targets=function(ctx,self)return ctx:friendly_minions(self)end}
function card.on_play(ctx,self,target)cardlib.effects.destroy(ctx, target);ctx:continue_with("destroy_random_enemy")end
function card.destroy_random_enemy(ctx,self)local pool={}for _,e in ipairs(ctx:enemy_characters(self))do if ctx:entity(e).type=="minion"then pool[#pool+1]=e end end;if #pool>0 then ctx:random_entity(pool,"sacrifice_enemy")end end
function card.sacrifice_enemy(ctx,self,target)cardlib.effects.destroy(ctx, target)end
return card
