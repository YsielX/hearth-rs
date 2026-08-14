local card={api_version=1,id="UNG_817",name="Tidal Surge",text="<b>Lifesteal</b>\nDeal $5 damage to a minion.",set="UNGORO",type="spell",class="shaman",rarity="common",spell_school="nature",cost=3,keywords={"lifesteal"},target_mode="required"}
function card.targets(ctx,self) return ctx:minions() end
function card.on_play(ctx,self,target) ctx:damage(target,5) end
return card
