local wanted={"taunt","divine_shield","lifesteal","windfury"}
local card={api_version=1,id="ICC_912",name="Corpsetaker",text="[x]<b>Battlecry:</b> Gain <b>Taunt</b> if your\ndeck has a <b>Taunt</b> minion.\nRepeat for <b>Divine Shield</b>,\n<b>Lifesteal</b>, <b>Windfury</b>.",set="ICECROWN",type="minion",rarity="epic",cost=4,attack=3,health=3,tags={"undead"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)for _,wanted_keyword in ipairs(wanted)do local found=false;for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="minion"then for _,k in ipairs(ctx:entity(e).keywords)do if k==wanted_keyword then found=true;break end end end;if found then break end end;if found then cardlib.effects.grant_keyword(ctx, self,wanted_keyword)end end end
return card
