local card={api_version=1,id="CS2_038",name="Ancestral Spirit",text="Give a minion \"<b>Deathrattle:</b> Resummon this minion.\"",set="EXPERT1",type="spell",class="shaman",rarity="rare",cost=2,target_mode="required",targets=function(ctx)return ctx:minions()end}
function card.on_play(ctx,self,target)ctx:attach_hook(target,"on_deathrattle","CS2_038");ctx:grant_keyword(target,"deathrattle")end
function card.on_deathrattle(ctx,self,pos)ctx:summon_at(ctx:controller(self),ctx:entity(self).card_id,pos)end
return card
