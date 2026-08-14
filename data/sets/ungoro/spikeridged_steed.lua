local card = {
    api_version = 1, id = "UNG_952", name = "Spikeridged Steed",
    text = "Give a minion +2/+6 and <b>Taunt</b>. When it dies, summon a Stegodon.",
    set = "UNGORO", type = "spell", class = "paladin", rarity = "rare",
    cost = 5, target_mode = "required", targets = function(ctx, self) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    ctx:buff(target, 2, 6)
    ctx:grant_keyword(target, "taunt")
    ctx:attach_deathrattle(target, "UNG_952")
    ctx:grant_keyword(target, "deathrattle")
end
function card.on_deathrattle(ctx, self, position)
    ctx:summon_at(ctx:controller(self), "UNG_810", position)
end
return card
