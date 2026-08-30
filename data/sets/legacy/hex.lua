local card={api_version=1,id="EX1_246", rarity = "free",name="Hex",text="Transform a minion into a 0/1 Frog with <b>Taunt</b>.",set="LEGACY",type="spell",class="shaman",spell_school="nature",cost=3,target_mode="required",targets=function(ctx)return ctx:minions()end,on_play=function(ctx,self,target)cardlib.effects.transform(ctx, target,"hexfrog")end}
card.tokens={{id="hexfrog",name="Frog",text="<b>Taunt</b>",set="LEGACY",type="minion",class="shaman",collectible=false,cost=0,attack=0,health=1,tags={"beast"},keywords={"taunt"}}}
return card
