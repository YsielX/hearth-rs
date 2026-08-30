local card = {
    api_version = 1, id = "UNG_952", name = "Spikeridged Steed",
    text = "Give a minion +2/+6 and <b>Taunt</b>. When it dies, summon a Stegodon.",
    set = "UNGORO", type = "spell", class = "paladin", rarity = "rare",
    cost = 5, target_mode = "required", targets = function(ctx, self) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    cardlib.effects.buff(ctx, target, 2, 6)
    cardlib.effects.grant_keyword(ctx, target, "taunt")
    ctx:attach_hook(target, "on_deathrattle", "UNG_952")
    cardlib.effects.grant_keyword(ctx, target, "deathrattle")
end
function card.on_deathrattle(ctx, self, position)
    cardlib.effects.summon_at(ctx, ctx:controller(self), "UNG_810", position)
end
return card
